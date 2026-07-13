<template>
	<div class="alert-root">
		<div class="window-bar" @mousedown="handleTitleBarMouseDown">
			<div class="window-title">{{ isResting ? "休息中" : "休息提醒" }}</div>
			<button class="window-btn" title="最小化" @click.stop="minimizeWindow">—</button>
		</div>

		<!-- 图标 -->
		<div class="icon-wrap" :class="isResting ? 'icon--rest' : 'icon--alert'">
			{{ isResting ? "🌿" : "🪑" }}
		</div>

		<!-- 标题区 -->
		<div class="title-area">
			<h1 class="title">{{ isResting ? "休息一下" : "该活动了！" }}</h1>
			<p class="subtitle">
				{{ isResting ? "计时结束后窗口将自动关闭" : "你已经高强度专注很久了，起来活动一下吧" }}
			</p>
		</div>

		<!-- 自动开始倒计时提示（仅 triggered 阶段） -->
		<div v-if="!isResting" class="auto-hint">
			{{ autoRestCountdown }} 秒后自动开始休息
		</div>

		<!-- 进度环 -->
		<div class="ring-wrap">
			<svg class="ring-svg" width="160" height="160" viewBox="0 0 160 160">
				<defs>
					<linearGradient id="rg" x1="0%" y1="0%" x2="100%" y2="100%">
						<stop offset="0%" stop-color="#7c3aed" />
						<stop offset="100%" stop-color="#a78bfa" />
					</linearGradient>
				</defs>
				<circle class="ring-track" cx="80" cy="80" r="70" stroke-width="8" />
				<circle
					class="ring-fill"
					cx="80"
					cy="80"
					r="70"
					stroke-width="8"
					:stroke-dasharray="circumference"
					:stroke-dashoffset="dashOffset"
				/>
			</svg>
			<div class="ring-center">
				<span class="ring-time">{{ displayTime }}</span>
				<span class="ring-label">{{ isResting ? "休息剩余" : "已超时" }}</span>
			</div>
		</div>

		<!-- 操作按钮 -->
		<div v-if="!isResting" class="actions">
			<button class="btn-primary" @click="startRest">🌿 开始休息</button>
			<div class="btn-row">
				<button class="btn-secondary" @click="extend">⏱ 再等一会</button>
				<button class="btn-ghost" @click="skip">跳过</button>
			</div>
		</div>

		<!-- 休息中：提示 + 取消按钮 -->
		<div v-else class="rest-content">
			<div class="tips-card">
				<div class="tip">👁 看向 6 米外，放松眼睛</div>
				<div class="tip">🧘 起立深呼吸几次</div>
				<div class="tip">💧 顺便去喝杯水</div>
			</div>
			<button class="btn-ghost" @click="skip">取消休息</button>
		</div>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";

const appWindow = getCurrentWebviewWindow();

const state = ref({
	phase: "triggered",
	sittingRemaining: 0,
	waterRemaining: 80 * 60,
	restRemaining: 5 * 60,
	sittingInterval: 50 * 60,
	waterInterval: 80 * 60,
	restDuration: 5 * 60,
	extendDuration: 5 * 60,
});

/** 自动开始休息的倒计时秒数 */
const autoRestCountdown = ref(10);
/** @type {number|null} */
let autoRestTimer = null;
/** @type {() => void | null} */
let unlistenTick = null;

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

const isResting = computed(() => state.value.phase === "resting");

const circumference = 2 * Math.PI * 70; // ≈ 439.8

const progressRatio = computed(() => {
	const s = state.value;
	if (s.phase === "resting") {
		return s.restRemaining / (s.restDuration || 5 * 60);
	}
	return Math.max(0, s.sittingRemaining / s.sittingInterval);
});

const dashOffset = computed(() => circumference * (1 - progressRatio.value));

const displayTime = computed(() => {
	const s = state.value;
	if (s.phase === "resting") return formatTime(s.restRemaining);
	return formatTime(s.sittingRemaining < 0 ? 0 : s.sittingRemaining);
});

// ── 自动开始休息倒计时 ───────────────────

function startAutoCountdown() {
	stopAutoCountdown();
	autoRestCountdown.value = 10;
	autoRestTimer = setInterval(() => {
		autoRestCountdown.value -= 1;
		if (autoRestCountdown.value <= 0) {
			stopAutoCountdown();
			startRest();
		}
	}, 1000);
}

function stopAutoCountdown() {
	if (autoRestTimer !== null) {
		clearInterval(autoRestTimer);
		autoRestTimer = null;
	}
}

/**
 * 顶栏空白区域触发系统拖动，避免误伤按钮点击。
 * @param {MouseEvent} event
 * @returns {void}
 */
function handleTitleBarMouseDown(event) {
	if (event.target.closest("button")) return;
	appWindow.startDragging();
}

// phase 切换到 triggered 时启动自动倒计时
watch(
	() => state.value.phase,
	(phase) => {
		if (phase === "triggered") {
			startAutoCountdown();
		} else {
			stopAutoCountdown();
		}
	},
);

// ── 操作 ─────────────────────────────────

function startRest() {
	stopAutoCountdown();
	invoke("user_action", { action: "start_rest" });
}

function skip() {
	stopAutoCountdown();
	invoke("user_action", { action: "skip" });
}

function extend() {
	stopAutoCountdown();
	invoke("user_action", { action: "extend" });
}

/** 最小化提醒窗口，便于稍后从任务栏恢复 */
function minimizeWindow() {
	appWindow.minimize();
}

// ── 生命周期 ─────────────────────────────

onMounted(async () => {
	state.value = await invoke("get_timer_state");

	// 初始状态若已是 triggered，立即启动
	if (state.value.phase === "triggered") {
		startAutoCountdown();
	}

	unlistenTick = await listen("timer-tick", ({ payload }) => {
		state.value = payload;
	});
});

onUnmounted(() => {
	stopAutoCountdown();
	unlistenTick?.();
});
</script>

<style scoped>
/* ── 根容器：浅色不透明背景，填满窗口 ── */
.alert-root {
	width: 100%;
	height: 100%;
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: flex-start;
	gap: 16px;
	padding: 12px 24px 28px;
	background: #f5f7ff;
	font-family: "Inter", "PingFang SC", "Microsoft YaHei", sans-serif;
	animation: fadeInUp 0.3s ease both;
	box-sizing: border-box;
}

.window-bar {
	width: 100%;
	display: flex;
	align-items: center;
	justify-content: space-between;
	cursor: move;
	user-select: none;
}

.window-title {
	font-size: 12px;
	font-weight: 600;
	color: #9ca3af;
	letter-spacing: 0.04em;
}

.window-btn {
	width: 28px;
	height: 24px;
	border: none;
	border-radius: 8px;
	background: transparent;
	color: #9ca3af;
	font-size: 16px;
	line-height: 1;
	cursor: pointer;
	transition:
		background 0.15s,
		color 0.15s;
}

.window-btn:hover {
	background: rgba(255, 255, 255, 0.7);
	color: #374151;
}

@keyframes fadeInUp {
	from {
		opacity: 0;
		transform: translateY(12px);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}

/* ── 图标 ── */
.icon-wrap {
	width: 56px;
	height: 56px;
	border-radius: 16px;
	display: flex;
	align-items: center;
	justify-content: center;
	font-size: 26px;
	flex-shrink: 0;
}

.icon--alert {
	background: #ede9fe;
	border: 1.5px solid #c4b5fd;
}

.icon--rest {
	background: #dcfce7;
	border: 1.5px solid #86efac;
}

/* ── 标题 ── */
.title-area {
	text-align: center;
}

.title {
	font-size: 20px;
	font-weight: 700;
	color: #111827;
	margin: 0 0 4px;
	letter-spacing: -0.02em;
}

.subtitle {
	font-size: 12px;
	color: #6b7280;
	line-height: 1.5;
	margin: 0;
}

/* ── 自动倒计时提示 ── */
.auto-hint {
	font-size: 12px;
	color: #7c3aed;
	background: #ede9fe;
	padding: 4px 12px;
	border-radius: 100px;
	font-weight: 500;
}

/* ── 进度环 ── */
.ring-wrap {
	position: relative;
	width: 160px;
	height: 160px;
	flex-shrink: 0;
}

.ring-svg {
	transform: rotate(-90deg);
}

.ring-track {
	fill: none;
	stroke: #e9eaf0;
}

.ring-fill {
	fill: none;
	stroke: url(#rg);
	stroke-linecap: round;
	transition: stroke-dashoffset 0.8s ease;
}

.ring-center {
	position: absolute;
	inset: 0;
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 2px;
}

.ring-time {
	font-family: "JetBrains Mono", "Fira Code", monospace;
	font-size: 30px;
	font-weight: 600;
	color: #1f2937;
	line-height: 1;
	letter-spacing: -0.02em;
}

.ring-label {
	font-size: 11px;
	color: #9ca3af;
	letter-spacing: 0.04em;
}

/* ── 操作按钮 ── */
.actions {
	width: 100%;
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.btn-row {
	display: flex;
	gap: 8px;
}

.btn-primary {
	width: 100%;
	padding: 11px 0;
	background: linear-gradient(135deg, #7c3aed, #a78bfa);
	color: #fff;
	border: none;
	border-radius: 10px;
	font-size: 14px;
	font-weight: 600;
	cursor: pointer;
	font-family: inherit;
	transition:
		transform 0.15s,
		box-shadow 0.15s;
	box-shadow: 0 4px 14px rgba(124, 58, 237, 0.35);
}

.btn-primary:hover {
	transform: translateY(-1px);
	box-shadow: 0 6px 20px rgba(124, 58, 237, 0.45);
}

.btn-secondary {
	flex: 1;
	padding: 9px 0;
	background: #ede9fe;
	color: #6d28d9;
	border: none;
	border-radius: 10px;
	font-size: 13px;
	font-weight: 500;
	cursor: pointer;
	font-family: inherit;
	transition: background 0.15s;
}

.btn-secondary:hover {
	background: #ddd6fe;
}

.btn-ghost {
	flex: 1;
	padding: 9px 0;
	background: #f3f4f6;
	color: #6b7280;
	border: none;
	border-radius: 10px;
	font-size: 13px;
	font-weight: 500;
	cursor: pointer;
	font-family: inherit;
	transition: background 0.15s;
}

.btn-ghost:hover {
	background: #e5e7eb;
	color: #374151;
}

/* ── 休息中内容 ── */
.rest-content {
	width: 100%;
	display: flex;
	flex-direction: column;
	gap: 10px;
}

.tips-card {
	background: #ffffff;
	border: 1px solid #e9eaf0;
	border-radius: 12px;
	padding: 12px 16px;
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.tip {
	font-size: 13px;
	color: #374151;
	line-height: 1.4;
}

/* 休息中取消按钮：全宽 */
.rest-content .btn-ghost {
	width: 100%;
}
</style>
