mod config;
mod timer;

use config::{
    autostart_supported, config_path, is_autostart_enabled, load_config, save_config,
    set_autostart_enabled, AppConfig, SettingsPayload,
};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, State, WindowEvent, Wry,
};
use timer::{AlertKind, AppTimer, Phase, SharedTimer};

const POPUP_WIDTH: f64 = 220.0;
const POPUP_HEIGHT: f64 = 180.0;

struct PauseMenuState {
    rest_item: CheckMenuItem<Wry>,
    water_item: CheckMenuItem<Wry>,
}

// ─────────────────────────────────────────
// Tauri Commands
// ─────────────────────────────────────────

/// 获取当前计时器快照，供前端初始化时同步状态
#[tauri::command]
fn get_timer_state(state: State<'_, SharedTimer>) -> timer::TimerSnapshot {
    state.lock().unwrap().snapshot()
}

/// 获取设置页所需的持久化配置与说明信息
#[tauri::command]
async fn get_settings() -> Result<SettingsPayload, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut config = load_config();
        config.autostart_enabled = is_autostart_enabled();

        Ok(SettingsPayload {
            config,
            config_path: config_path().display().to_string(),
            autostart_supported: autostart_supported(),
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

/// 用户操作：start_rest / skip / extend / dismiss_water / reset_xxx / toggle_xxx_pause
#[tauri::command]
fn user_action(action: String, state: State<'_, SharedTimer>, app: AppHandle) {
    let should_hide_alert = {
        let mut t = state.lock().unwrap();
        match action.as_str() {
            // 开始休息：进入休息倒计时，alert 窗口保持显示
            "start_rest" => {
                t.rest_timer_paused = false;
                t.phase = Phase::Resting;
                t.rest_remaining = t.rest_duration as i64;
            }
            // 跳过 / 取消休息：重置久坐计时，关闭 alert 窗口
            "skip" => {
                t.rest_timer_paused = false;
                t.phase = Phase::Running;
                t.sitting_remaining = t.sitting_interval as i64;
                t.dismiss_water_alert();
            }
            // 延长：重置为短计时器，关闭 alert 窗口
            "extend" => {
                t.rest_timer_paused = false;
                t.phase = Phase::Running;
                t.sitting_remaining = t.extend_duration as i64;
                t.dismiss_water_alert();
            }
            // 分别暂停/恢复休息与喝水计时
            "toggle_rest_pause" => {
                t.toggle_rest_timer_paused();
            }
            "toggle_water_pause" => {
                t.toggle_water_timer_paused();
            }
            // 分别重置休息与喝水计时
            "reset_rest_timer" => {
                t.reset_rest_timer();
            }
            "reset_water_timer" => {
                t.reset_water_timer();
            }
            // 手动关闭喝水短提醒
            "dismiss_water" => {
                t.dismiss_water_alert();
            }
            _ => {}
        }

        t.active_alert().is_none()
    };

    if should_hide_alert {
        hide_alert(&app);
    }

    let snapshot = state.lock().unwrap().snapshot();
    let menu_state = app.state::<PauseMenuState>();
    sync_pause_menu_state(
        &menu_state.rest_item,
        &menu_state.water_item,
        snapshot.rest_timer_paused,
        snapshot.water_timer_paused,
    );
    let _ = app.emit("timer-tick", snapshot);
}

/// 前端请求隐藏 popup 窗口（blur 事件触发）
#[tauri::command]
fn hide_popup(app: AppHandle) {
    if let Some(w) = app.get_webview_window("popup") {
        let _ = w.hide();
    }
}

/// 打开托盘计时弹窗
#[tauri::command]
fn show_popup(app: AppHandle) {
    position_popup_bottom_right(&app);
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.set_size(tauri::LogicalSize::new(POPUP_WIDTH, POPUP_HEIGHT));
        let _ = popup.unminimize();
        let _ = popup.show();
        let _ = popup.set_focus();
    }
}

/// 打开设置窗口
#[tauri::command]
fn open_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 打开配置目录，便于用户直接查看或备份配置
#[tauri::command]
fn open_config_dir() -> Result<(), String> {
    let config_dir = config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| "无法解析配置目录".to_string())?;

    fs::create_dir_all(&config_dir).map_err(|err| err.to_string())?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|err| err.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("当前平台暂不支持直接打开配置目录".to_string())
    }
}

/// 设置窗口关闭时改为隐藏，避免托盘再次打开时找不到已销毁窗口
fn register_settings_close_handler(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let settings_window = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = settings_window.hide();
            }
        });
    }
}

/// 更新计时器配置（来自设置页面，单位：秒）
#[tauri::command]
async fn set_timer_config(
    silent_start: Option<bool>,
    auto_rest_secs: Option<u64>,
    sitting_interval_secs: Option<u64>,
    water_interval_secs: Option<u64>,
    rest_duration_secs: Option<u64>,
    extend_duration_secs: Option<u64>,
    rest_enabled: Option<bool>,
    water_enabled: Option<bool>,
    autostart_enabled: Option<bool>,
    app: AppHandle,
    state: State<'_, SharedTimer>,
) -> Result<(), String> {
    let (snapshot, should_hide_alert, next_config) = {
        let mut t = state.lock().unwrap();
        if let Some(v) = auto_rest_secs {
            t.auto_rest_secs = v;
        }
        if let Some(v) = sitting_interval_secs {
            t.sitting_interval = v;
            // 若剩余超过新上限则截断，避免下次提醒遥遥无期
            if t.sitting_remaining > v as i64 {
                t.sitting_remaining = v as i64;
            }
        }
        if let Some(v) = water_interval_secs {
            t.water_interval = v;
            if t.water_remaining > v as i64 {
                t.water_remaining = v as i64;
            }
        }
        if let Some(v) = rest_duration_secs {
            t.rest_duration = v;
        }
        if let Some(v) = extend_duration_secs {
            t.extend_duration = v;
        }
        if let Some(v) = rest_enabled {
            t.rest_enabled = v;
            if !v && matches!(t.active_alert(), Some(AlertKind::Sitting | AlertKind::Resting)) {
                t.phase = Phase::Running;
                t.rest_remaining = 0;
                t.sitting_remaining = t.sitting_interval as i64;
            }
        }
        if let Some(v) = water_enabled {
            t.water_enabled = v;
            if !v {
                t.dismiss_water_alert();
                t.water_remaining = t.water_interval as i64;
            }
        }

        let snapshot = t.snapshot();
        let should_hide_alert = snapshot.active_alert.is_none();
        let next_config = AppConfig {
            silent_start: silent_start.unwrap_or(false),
            auto_rest_secs: t.auto_rest_secs,
            sitting_interval_secs: t.sitting_interval,
            water_interval_secs: t.water_interval,
            rest_duration_secs: t.rest_duration,
            extend_duration_secs: t.extend_duration,
            rest_enabled: t.rest_enabled,
            water_enabled: t.water_enabled,
            autostart_enabled: autostart_enabled.unwrap_or(false),
        };

        (snapshot, should_hide_alert, next_config)
    };

    tauri::async_runtime::spawn_blocking(move || {
        let mut config = load_config();
        if let Some(v) = silent_start {
            config.silent_start = v;
        }
        config.auto_rest_secs = next_config.auto_rest_secs;
        config.sitting_interval_secs = next_config.sitting_interval_secs;
        config.water_interval_secs = next_config.water_interval_secs;
        config.rest_duration_secs = next_config.rest_duration_secs;
        config.extend_duration_secs = next_config.extend_duration_secs;
        config.rest_enabled = next_config.rest_enabled;
        config.water_enabled = next_config.water_enabled;

        if let Some(v) = autostart_enabled {
            set_autostart_enabled(v)?;
            config.autostart_enabled = v;
        }

        save_config(&config)
    })
    .await
    .map_err(|err| err.to_string())??;

    if should_hide_alert {
        hide_alert(&app);
    }

    let _ = app.emit("timer-tick", snapshot);
    Ok(())
}

// ─────────────────────────────────────────
// 辅助函数
// ─────────────────────────────────────────

fn hide_alert(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("alert") {
        let _ = w.hide();
    }
}

fn show_alert(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("alert") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 同步托盘菜单中的暂停勾选状态
fn sync_pause_menu_state<R: Runtime>(
    rest_item: &CheckMenuItem<R>,
    water_item: &CheckMenuItem<R>,
    rest_paused: bool,
    water_paused: bool,
) {
    let _ = rest_item.set_checked(rest_paused);
    let _ = water_item.set_checked(water_paused);
}

/// 切换指定计时器的暂停状态，并立即广播给前端刷新状态
fn toggle_timer_pause<R: Runtime>(
    app: &AppHandle,
    timer: &SharedTimer,
    rest_item: &CheckMenuItem<R>,
    water_item: &CheckMenuItem<R>,
    kind: &str,
) {
    let snapshot = {
        let mut state = timer.lock().unwrap();
        match kind {
            "rest" => state.toggle_rest_timer_paused(),
            "water" => state.toggle_water_timer_paused(),
            _ => return,
        }
        state.snapshot()
    };

    sync_pause_menu_state(
        rest_item,
        water_item,
        snapshot.rest_timer_paused,
        snapshot.water_timer_paused,
    );
    let _ = app.emit("timer-tick", snapshot);
}

/// 定位 popup 窗口到屏幕右下角（托盘区域上方）
fn position_popup_bottom_right(app: &AppHandle) {
    if let Some(popup) = app.get_webview_window("popup") {
        if let Ok(Some(monitor)) = popup.primary_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size();
            let logical_w = size.width as f64 / scale;
            let logical_h = size.height as f64 / scale;
            // 弹窗尺寸由常量统一控制，避免样式和窗口配置不一致
            let x = logical_w - POPUP_WIDTH - 12.0;
            let y = logical_h - POPUP_HEIGHT - 12.0 - 48.0;
            let _ = popup.set_position(tauri::LogicalPosition::new(x, y));
        }
    }
}

// ─────────────────────────────────────────
// 后台计时器 tokio 任务
// ─────────────────────────────────────────

fn start_timer_loop(app: AppHandle, timer: SharedTimer) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let (snapshot, side_effect) = {
                let mut t = timer.lock().unwrap();

                let mut effect = None;

                if t.tick_water_alert() && t.active_alert().is_none() {
                    effect = Some("water-reminder-ended");
                }

                let phase_effect = match t.phase {
                    Phase::Resting => {
                        if t.rest_timer_paused {
                            None
                        } else {
                            t.rest_remaining -= 1;
                            if t.rest_remaining <= 0 {
                                t.phase = Phase::Running;
                                t.sitting_remaining = t.sitting_interval as i64;
                                Some("rest-ended")
                            } else {
                                None
                            }
                        }
                    }

                    Phase::Triggered => {
                        if t.water_enabled && !t.water_timer_paused {
                            t.water_remaining -= 1;
                        }
                        if t.water_enabled && t.water_remaining <= 0 {
                            t.water_remaining = t.water_interval as i64;
                            None
                        } else {
                            None
                        }
                    }

                    Phase::Running => {
                        if !t.rest_timer_paused {
                            t.sitting_remaining -= 1;
                        }
                        if t.water_enabled && !t.water_timer_paused {
                            t.water_remaining -= 1;
                        }

                        if t.water_enabled && t.water_remaining <= 0 {
                            t.water_remaining = t.water_interval as i64;
                            t.show_water_alert(5);
                            if t.rest_enabled && t.sitting_remaining <= 0 {
                                t.phase = Phase::Triggered;
                                Some("sitting-triggered")
                            } else {
                                Some("water-reminder")
                            }
                        } else if t.sitting_remaining <= 0 && t.rest_enabled {
                            t.phase = Phase::Triggered;
                            Some("sitting-triggered")
                        } else if t.sitting_remaining <= 0 {
                            t.phase = Phase::Running;
                            t.sitting_remaining = t.sitting_interval as i64;
                            None
                        } else {
                            None
                        }
                    }
                };

                let effect = phase_effect.or(effect);
                (t.snapshot(), effect)
            };

            if let Some(effect) = side_effect {
                match effect {
                    "sitting-triggered" => show_alert(&app),
                    "rest-ended" => hide_alert(&app),
                    "water-reminder" => show_alert(&app),
                    "water-reminder-ended" => hide_alert(&app),
                    _ => {}
                }
            }

            let _ = app.emit("timer-tick", snapshot);
        }
    });
}

// ─────────────────────────────────────────
// 应用入口
// ─────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = load_config();
    let mut timer = AppTimer::new();
    timer.apply_config(&initial_config);
    let timer_state: SharedTimer = Arc::new(Mutex::new(timer));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(timer_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            get_settings,
            user_action,
            hide_popup,
            show_popup,
            open_settings,
            open_config_dir,
            set_timer_config,
        ])
        .setup(move |app| {
            register_settings_close_handler(app.handle());

            let show_time_item =
                MenuItem::with_id(app, "show_time", "显示时间", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let pause_rest_item = CheckMenuItemBuilder::with_id("pause_rest", "暂停休息计时")
                .checked(false)
                .build(app)?;
            let pause_water_item = CheckMenuItemBuilder::with_id("pause_water", "暂停喝水计时")
                .checked(false)
                .build(app)?;
            let mode_submenu = SubmenuBuilder::new(app, "暂停计时")
                .item(&pause_rest_item)
                .item(&pause_water_item)
                .build()?;
            app.manage(PauseMenuState {
                rest_item: pause_rest_item.clone(),
                water_item: pause_water_item.clone(),
            });

            // 调试菜单项仅在 debug 构建中出现
            let menu = if cfg!(debug_assertions) {
                let debug_sitting = MenuItem::with_id(
                    app,
                    "debug_sitting",
                    "[调试] 立即触发久坐提醒",
                    true,
                    None::<&str>,
                )?;
                let debug_water = MenuItem::with_id(
                    app,
                    "debug_water",
                    "[调试] 立即触发喝水提醒",
                    true,
                    None::<&str>,
                )?;
                MenuBuilder::new(app)
                    .items(&[
                        &show_time_item,
                        &settings_item,
                        &mode_submenu,
                        &debug_sitting,
                        &debug_water,
                        &quit_item,
                    ])
                    .build()?
            } else {
                MenuBuilder::new(app)
                    .items(&[&show_time_item, &settings_item, &mode_submenu, &quit_item])
                    .build()?
            };

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("歇会儿")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(popup) = app.get_webview_window("popup") {
                            if popup.is_visible().unwrap_or(false) {
                                let _ = popup.hide();
                            } else {
                                show_popup(app.clone());
                            }
                        }
                    }
                })
                .on_menu_event({
                    let pause_rest_item = pause_rest_item.clone();
                    let pause_water_item = pause_water_item.clone();
                    move |app, event| match event.id.as_ref() {
                        "show_time" => show_popup(app.clone()),
                        "settings" => open_settings(app.clone()),
                        "pause_rest" => {
                            let state = app.state::<SharedTimer>();
                            toggle_timer_pause(
                                app,
                                &state.inner().clone(),
                                &pause_rest_item,
                                &pause_water_item,
                                "rest",
                            );
                        }
                        "pause_water" => {
                            let state = app.state::<SharedTimer>();
                            toggle_timer_pause(
                                app,
                                &state.inner().clone(),
                                &pause_rest_item,
                                &pause_water_item,
                                "water",
                            );
                        }
                        // 调试：仅 debug 构建可触发，生产构建菜单中无此项
                        "debug_sitting" => {
                            let state = app.state::<SharedTimer>();
                            let mut t = state.lock().unwrap();
                            t.sitting_remaining = 0;
                        }
                        "debug_water" => {
                            let state = app.state::<SharedTimer>();
                            let mut t = state.lock().unwrap();
                            t.show_water_alert(5);
                            drop(t);
                            show_alert(app);
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;

            if !initial_config.silent_start {
                open_settings(app.handle().clone());
            }

            let app_handle = app.handle().clone();
            start_timer_loop(app_handle, timer_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
