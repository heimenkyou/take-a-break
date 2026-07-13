use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 计时器运行阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    /// 正常倒计时
    Running,
    /// 久坐提醒已触发，等待用户操作（主计时暂停）
    Triggered,
    /// 用户点击"开始休息"，正向休息倒计时中
    Resting,
    /// 用户手动暂停全部提醒
    Paused,
}

/// 当前提醒窗口承载的提醒类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AlertKind {
    /// 久坐提醒，等待用户决定开始休息、延长或跳过
    Sitting,
    /// 已进入休息倒计时
    Resting,
    /// 短暂喝水提醒
    Water,
}

/// 推送给前端的计时器快照
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub phase: Phase,
    pub active_alert: Option<AlertKind>,
    /// 久坐倒计时剩余秒数（Running/Triggered 阶段有意义）
    pub sitting_remaining: i64,
    /// 喝水倒计时剩余秒数
    pub water_remaining: i64,
    /// 休息倒计时剩余秒数（仅 Resting 阶段有意义）
    pub rest_remaining: i64,
    /// 喝水提醒窗口剩余展示秒数
    pub water_alert_remaining: i64,
    /// 自动开始休息倒计时秒数
    pub auto_rest_secs: u64,
    /// 当前久坐间隔配置
    pub sitting_interval: u64,
    /// 当前喝水间隔配置
    pub water_interval: u64,
    /// 休息时长配置
    pub rest_duration: u64,
    /// 延长等待时长配置
    pub extend_duration: u64,
    /// 是否启用休息提醒
    pub rest_enabled: bool,
    /// 是否启用喝水提醒
    pub water_enabled: bool,
}

/// 计时器内部状态，由 tokio 任务持有并修改
pub struct AppTimer {
    pub phase: Phase,
    /// 暂停前所处阶段，恢复时用于回到原状态
    paused_phase: Option<Phase>,
    /// 喝水提醒剩余展示秒数，0 表示当前无短提醒窗口
    water_alert_remaining: i64,
    pub sitting_remaining: i64,
    pub water_remaining: i64,
    pub rest_remaining: i64,
    // ---- 配置 ----
    /// 自动开始休息倒计时（秒），默认 10 秒
    pub auto_rest_secs: u64,
    /// 久坐提醒间隔（秒），默认 50 分钟
    pub sitting_interval: u64,
    /// 喝水提醒间隔（秒），默认 80 分钟
    pub water_interval: u64,
    /// 休息时长（秒），默认 5 分钟
    pub rest_duration: u64,
    /// 延长后的短计时时长（秒），默认 5 分钟
    pub extend_duration: u64,
    /// 是否启用休息提醒
    pub rest_enabled: bool,
    /// 是否启用喝水提醒
    pub water_enabled: bool,
}

impl AppTimer {
    pub fn new() -> Self {
        Self {
            phase: Phase::Running,
            paused_phase: None,
            water_alert_remaining: 0,
            sitting_remaining: (50 * 60) as i64,
            water_remaining: (80 * 60) as i64,
            rest_remaining: 0,
            auto_rest_secs: 10,
            sitting_interval: 50 * 60,
            water_interval: 80 * 60,
            rest_duration: 5 * 60,
            extend_duration: 5 * 60,
            rest_enabled: true,
            water_enabled: true,
        }
    }

    pub fn snapshot(&self) -> TimerSnapshot {
        TimerSnapshot {
            phase: self.phase.clone(),
            active_alert: self.active_alert(),
            sitting_remaining: self.sitting_remaining,
            water_remaining: self.water_remaining,
            rest_remaining: self.rest_remaining,
            water_alert_remaining: self.water_alert_remaining,
            auto_rest_secs: self.auto_rest_secs,
            sitting_interval: self.sitting_interval,
            water_interval: self.water_interval,
            rest_duration: self.rest_duration,
            extend_duration: self.extend_duration,
            rest_enabled: self.rest_enabled,
            water_enabled: self.water_enabled,
        }
    }

    /// 设置暂停模式，并在恢复时回到暂停前的阶段
    pub fn set_paused(&mut self, paused: bool) {
        if paused {
            if self.phase != Phase::Paused {
                self.paused_phase = Some(self.phase.clone());
                self.phase = Phase::Paused;
            }
            return;
        }

        if self.phase == Phase::Paused {
            self.phase = self.paused_phase.take().unwrap_or(Phase::Running);
        }
    }

    /// 当前是否应展示提醒窗口
    pub fn active_alert(&self) -> Option<AlertKind> {
        if self.water_alert_remaining > 0 {
            return Some(AlertKind::Water);
        }

        match self.phase {
            Phase::Triggered => Some(AlertKind::Sitting),
            Phase::Resting => Some(AlertKind::Resting),
            _ => None,
        }
    }

    /// 打开短暂的喝水提醒窗口
    pub fn show_water_alert(&mut self, seconds: i64) {
        self.water_alert_remaining = seconds.max(0);
    }

    /// 关闭喝水提醒窗口
    pub fn dismiss_water_alert(&mut self) {
        self.water_alert_remaining = 0;
    }

    /// 推进一步喝水短提醒倒计时；返回 true 表示刚刚结束
    pub fn tick_water_alert(&mut self) -> bool {
        if self.water_alert_remaining <= 0 {
            return false;
        }

        self.water_alert_remaining -= 1;
        self.water_alert_remaining <= 0
    }
}

/// 跨线程共享的计时器状态
pub type SharedTimer = Arc<Mutex<AppTimer>>;
