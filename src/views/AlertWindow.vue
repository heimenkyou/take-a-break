<template>
  <div class="alert-root anim-fade-in">
    <div class="alert-card glass-card">
      <!-- 顶部装饰光晕 -->
      <div class="glow-orb" />

      <!-- 图标区 -->
      <div class="icon-wrap" :class="iconWrapClass">
        <span class="icon">{{ phaseIcon }}</span>
      </div>

      <!-- 标题区 -->
      <div class="title-area">
        <h1 class="title">{{ title }}</h1>
        <p class="subtitle">{{ subtitle }}</p>
      </div>

      <!-- 倒计时 / 进度区 -->
      <div class="countdown-area">
        <svg class="progress-ring" width="120" height="120" viewBox="0 0 120 120">
          <defs>
            <linearGradient id="ringGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stop-color="#7c6af7" />
              <stop offset="100%" stop-color="#a78bfa" />
            </linearGradient>
          </defs>
          <circle class="progress-ring__track" cx="60" cy="60" r="52" stroke-width="5" />
          <circle
            class="progress-ring__fill"
            cx="60" cy="60" r="52" stroke-width="5"
            :stroke-dasharray="circumference"
            :stroke-dashoffset="dashOffset"
          />
        </svg>
        <div class="countdown-overlay">
          <span class="countdown" :style="{ fontSize: '34px' }">{{ displayTime }}</span>
          <span class="countdown-label">{{ countdownLabel }}</span>
        </div>
      </div>

      <!-- 操作按钮区 -->
      <div v-if="state.phase === 'triggered'" class="actions anim-fade-in-up">
        <button class="btn btn-primary" style="width: 100%" @click="startRest">
          🌿 开始休息 · 5 分钟
        </button>
        <div class="actions-row">
          <button class="btn btn-ghost" style="flex: 1" @click="extend">
            ⏱ 再等 5 分钟
          </button>
          <button class="btn btn-danger-ghost" style="flex: 1" @click="skip">
            ⚡ 跳过
          </button>
        </div>
      </div>

      <!-- 休息中状态 -->
      <div v-else-if="state.phase === 'resting'" class="rest-tip anim-fade-in">
        <div class="rest-tips-list">
          <div class="tip-item">👁 看向 6 米外，放松眼部肌肉</div>
          <div class="tip-item">🧘 起立做几次深呼吸</div>
          <div class="tip-item">💧 顺便去接杯水</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";

const state = ref({
	phase: "triggered",
	sittingRemaining: 0,
	waterRemaining: 80 * 60,
	restRemaining: 5 * 60,
	sittingInterval: 50 * 60,
	waterInterval: 80 * 60,
});

// ── 格式化 ──────────────────────────────

/** @param {number} secs */
function formatTime(secs) {
	if (secs < 0) secs = 0;
	const m = Math.floor(secs / 60)
		.toString()
		.padStart(2, "0");
	const s = (secs % 60).toString().padStart(2, "0");
	return `${m}:${s}`;
}

// ── 计算属性 ─────────────────────────────

const circumference = 2 * Math.PI * 52; // ≈ 326.7

const progressRatio = computed(() => {
	const s = state.value;
	if (s.phase === "resting") {
		return s.restRemaining / (5 * 60);
	}
	// triggered: 显示已等待比例（从满格开始递减）
	return Math.max(0, s.sittingRemaining / s.sittingInterval);
});

const dashOffset = computed(() => circumference * (1 - progressRatio.value));

const displayTime = computed(() => {
	const s = state.value;
	if (s.phase === "resting") return formatTime(s.restRemaining);
	return formatTime(s.sittingRemaining < 0 ? 0 : s.sittingRemaining);
});

const countdownLabel = computed(() =>
	state.value.phase === "resting" ? "休息剩余" : "已超时",
);

const phaseIcon = computed(() =>
	state.value.phase === "resting" ? "🌿" : "⏰",
);

const iconWrapClass = computed(() =>
	state.value.phase === "resting" ? "icon-wrap--rest" : "icon-wrap--alert",
);

const title = computed(() =>
	state.value.phase === "resting" ? "休息一下" : "该活动了！",
);

const subtitle = computed(() =>
	state.value.phase === "resting"
		? "计时结束后窗口将自动关闭，下次提醒重新开始"
		: "你已经高强度专注了很久，起来活动一下吧",
);

// ── 操作 ─────────────────────────────────

function startRest() {
	invoke("user_action", { action: "start_rest" });
}

function skip() {
	invoke("user_action", { action: "skip" });
}

function extend() {
	invoke("user_action", { action: "extend" });
}

// ── 生命周期 ─────────────────────────────

let unlistenTick = null;

onMounted(async () => {
	state.value = await invoke("get_timer_state");

	unlistenTick = await listen("timer-tick", ({ payload }) => {
		state.value = payload;
	});
});

onUnmounted(() => {
	unlistenTick?.();
});
</script>

<style scoped>
.alert-root {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
}

.alert-card {
  width: 440px;
  border-radius: 28px;
  padding: 32px 28px 28px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  position: relative;
  overflow: hidden;
  animation: cardEntrance 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) both;
}

@keyframes cardEntrance {
  from {
    opacity: 0;
    transform: scale(0.88) translateY(20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

/* 顶部紫色光晕 */
.glow-orb {
  position: absolute;
  top: -60px;
  left: 50%;
  transform: translateX(-50%);
  width: 240px;
  height: 120px;
  background: radial-gradient(ellipse, rgba(124, 106, 247, 0.4), transparent 70%);
  pointer-events: none;
}

/* ── 图标 ── */
.icon-wrap {
  width: 60px;
  height: 60px;
  border-radius: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
}

.icon-wrap--alert {
  background: linear-gradient(135deg, rgba(124, 106, 247, 0.2), rgba(167, 139, 250, 0.1));
  border: 1px solid rgba(124, 106, 247, 0.3);
  animation: pulse-glow 2.5s ease-in-out infinite;
}

.icon-wrap--rest {
  background: linear-gradient(135deg, rgba(74, 222, 128, 0.15), rgba(74, 222, 128, 0.05));
  border: 1px solid rgba(74, 222, 128, 0.25);
}

/* ── 标题 ── */
.title-area {
  text-align: center;
}

.title {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 6px;
  letter-spacing: -0.02em;
}

.subtitle {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.5;
  max-width: 300px;
}

/* ── 进度环 ── */
.countdown-area {
  position: relative;
  width: 120px;
  height: 120px;
}

.countdown-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
}

.countdown-label {
  font-size: 10px;
  color: var(--color-text-muted);
  letter-spacing: 0.05em;
}

/* ── 按钮区 ── */
.actions {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.actions-row {
  display: flex;
  gap: 10px;
}

/* ── 休息提示 ── */
.rest-tip {
  width: 100%;
}

.rest-tips-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.tip-item {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}
</style>
