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

/// 推送给前端的计时器快照
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub phase: Phase,
    /// 久坐倒计时剩余秒数（Running/Triggered 阶段有意义）
    pub sitting_remaining: i64,
    /// 喝水倒计时剩余秒数
    pub water_remaining: i64,
    /// 休息倒计时剩余秒数（仅 Resting 阶段有意义）
    pub rest_remaining: i64,
    /// 当前久坐间隔配置
    pub sitting_interval: u64,
    /// 当前喝水间隔配置
    pub water_interval: u64,
    /// 休息时长配置
    pub rest_duration: u64,
    /// 延长等待时长配置
    pub extend_duration: u64,
}

/// 计时器内部状态，由 tokio 任务持有并修改
pub struct AppTimer {
    pub phase: Phase,
    pub sitting_remaining: i64,
    pub water_remaining: i64,
    pub rest_remaining: i64,
    // ---- 配置 ----
    /// 久坐提醒间隔（秒），默认 50 分钟
    pub sitting_interval: u64,
    /// 喝水提醒间隔（秒），默认 80 分钟
    pub water_interval: u64,
    /// 休息时长（秒），默认 5 分钟
    pub rest_duration: u64,
    /// 延长后的短计时时长（秒），默认 5 分钟
    pub extend_duration: u64,
}

impl AppTimer {
    pub fn new() -> Self {
        Self {
            phase: Phase::Running,
            sitting_remaining: (50 * 60) as i64,
            water_remaining: (80 * 60) as i64,
            rest_remaining: 0,
            sitting_interval: 50 * 60,
            water_interval: 80 * 60,
            rest_duration: 5 * 60,
            extend_duration: 5 * 60,
        }
    }

    pub fn snapshot(&self) -> TimerSnapshot {
        TimerSnapshot {
            phase: self.phase.clone(),
            sitting_remaining: self.sitting_remaining,
            water_remaining: self.water_remaining,
            rest_remaining: self.rest_remaining,
            sitting_interval: self.sitting_interval,
            water_interval: self.water_interval,
            rest_duration: self.rest_duration,
            extend_duration: self.extend_duration,
        }
    }
}

/// 跨线程共享的计时器状态
pub type SharedTimer = Arc<Mutex<AppTimer>>;
