mod timer;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_notification::NotificationExt;
use timer::{AppTimer, Phase, SharedTimer};

// ─────────────────────────────────────────
// Tauri Commands
// ─────────────────────────────────────────

/// 获取当前计时器快照，供前端初始化时同步状态
#[tauri::command]
fn get_timer_state(state: State<'_, SharedTimer>) -> timer::TimerSnapshot {
    state.lock().unwrap().snapshot()
}

/// 用户操作：start_rest / skip / extend / pause_toggle
#[tauri::command]
fn user_action(action: String, state: State<'_, SharedTimer>, app: AppHandle) {
    let mut t = state.lock().unwrap();
    match action.as_str() {
        // 开始休息：进入休息倒计时，alert 窗口保持显示
        "start_rest" => {
            t.phase = Phase::Resting;
            t.rest_remaining = t.rest_duration as i64;
        }
        // 跳过 / 取消休息：重置久坐计时，关闭 alert 窗口
        "skip" => {
            t.phase = Phase::Running;
            t.sitting_remaining = t.sitting_interval as i64;
            drop(t);
            hide_alert(&app);
        }
        // 延长：重置为短计时器，关闭 alert 窗口
        "extend" => {
            t.phase = Phase::Running;
            t.sitting_remaining = t.extend_duration as i64;
            drop(t);
            hide_alert(&app);
        }
        // 暂停/恢复
        "pause_toggle" => {
            t.phase = if t.phase == Phase::Paused {
                Phase::Running
            } else {
                Phase::Paused
            };
        }
        _ => {}
    }
}

/// 前端请求隐藏 popup 窗口（blur 事件触发）
#[tauri::command]
fn hide_popup(app: AppHandle) {
    if let Some(w) = app.get_webview_window("popup") {
        let _ = w.hide();
    }
}

/// 打开设置窗口
#[tauri::command]
fn open_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 更新计时器配置（来自设置页面，单位：秒）
#[tauri::command]
fn set_timer_config(
    sitting_interval_secs: Option<u64>,
    water_interval_secs: Option<u64>,
    rest_duration_secs: Option<u64>,
    extend_duration_secs: Option<u64>,
    state: State<'_, SharedTimer>,
) {
    let mut t = state.lock().unwrap();
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
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 定位 popup 窗口到屏幕右下角（托盘区域上方）
fn position_popup_bottom_right(app: &AppHandle) {
    if let Some(popup) = app.get_webview_window("popup") {
        if let Ok(Some(monitor)) = popup.primary_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size();
            let logical_w = size.width as f64 / scale;
            let logical_h = size.height as f64 / scale;
            // 弹窗尺寸 220×180，距边缘 12px，任务栏高度约 48px
            let x = logical_w - 220.0 - 12.0;
            let y = logical_h - 180.0 - 12.0 - 48.0;
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

                let effect = match t.phase {
                    Phase::Paused => None,

                    Phase::Resting => {
                        t.rest_remaining -= 1;
                        if t.rest_remaining <= 0 {
                            t.phase = Phase::Running;
                            t.sitting_remaining = t.sitting_interval as i64;
                            Some("rest-ended")
                        } else {
                            None
                        }
                    }

                    Phase::Triggered => {
                        t.water_remaining -= 1;
                        if t.water_remaining <= 0 {
                            t.water_remaining = t.water_interval as i64;
                            Some("water-reminder")
                        } else {
                            None
                        }
                    }

                    Phase::Running => {
                        t.sitting_remaining -= 1;
                        t.water_remaining -= 1;

                        if t.water_remaining <= 0 {
                            t.water_remaining = t.water_interval as i64;
                            if t.sitting_remaining <= 0 {
                                t.phase = Phase::Triggered;
                                Some("sitting-triggered")
                            } else {
                                Some("water-reminder")
                            }
                        } else if t.sitting_remaining <= 0 {
                            t.phase = Phase::Triggered;
                            Some("sitting-triggered")
                        } else {
                            None
                        }
                    }
                };

                (t.snapshot(), effect)
            };

            if let Some(effect) = side_effect {
                match effect {
                    "sitting-triggered" => show_alert(&app),
                    "rest-ended" => hide_alert(&app),
                    "water-reminder" => {
                        let _ = app
                            .notification()
                            .builder()
                            .title("歇会儿 💧")
                            .body("记得喝水！每次 200ml 左右")
                            .show();
                    }
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
    let timer_state: SharedTimer = Arc::new(Mutex::new(AppTimer::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(timer_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            user_action,
            hide_popup,
            open_settings,
            set_timer_config,
        ])
        .setup(|app| {
            let pause_item =
                MenuItem::with_id(app, "pause", "暂停提醒", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

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
                Menu::with_items(
                    app,
                    &[
                        &settings_item,
                        &pause_item,
                        &debug_sitting,
                        &debug_water,
                        &quit_item,
                    ],
                )?
            } else {
                Menu::with_items(app, &[&settings_item, &pause_item, &quit_item])?
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
                                position_popup_bottom_right(app);
                                let _ = popup.show();
                                let _ = popup.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(w) = app.get_webview_window("settings") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "pause" => {
                        let state = app.state::<SharedTimer>();
                        let mut t = state.lock().unwrap();
                        t.phase = if t.phase == Phase::Paused {
                            Phase::Running
                        } else {
                            Phase::Paused
                        };
                    }
                    // 调试：仅 debug 构建可触发，生产构建菜单中无此项
                    "debug_sitting" => {
                        let state = app.state::<SharedTimer>();
                        let mut t = state.lock().unwrap();
                        t.sitting_remaining = 0;
                    }
                    "debug_water" => {
                        let _ = app
                            .notification()
                            .builder()
                            .title("歇会儿 💧")
                            .body("记得喝水！每次 200ml 左右")
                            .show();
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            start_timer_loop(app_handle, timer_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
