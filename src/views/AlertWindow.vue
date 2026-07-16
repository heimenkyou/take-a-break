<template>
	<div class="alert-root">
		<div class="window-bar" @mousedown="handleTitleBarMouseDown">
			<div class="window-title">{{ windowTitle }}</div>
			<button class="window-btn" title="最小化" @click.stop="minimizeWindow">—</button>
		</div>

		<!-- 图标 -->
		<div class="icon-wrap" :class="iconClass">
			{{ iconText }}
		</div>

		<!-- 标题区 -->
		<div class="title-area">
			<h1 class="title">{{ titleText }}</h1>
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
				<span class="ring-label">{{ ringLabel }}</span>
			</div>
		</div>

		<!-- 操作按钮 -->
		<div v-if="isSittingAlert" class="actions">
			<button class="btn-primary" @click="startRest">立即开始休息</button>
			<div class="btn-row">
				<button class="btn-secondary" @click="extend">稍后再说</button>
				<button class="btn-ghost" @click="skip">跳过本次</button>
			</div>
		</div>

		<!-- 休息中：提示 + 取消按钮 -->
		<div v-else-if="isRestingAlert" class="rest-content">
			<div class="tips-card">
				<div class="tip">👁 看向 6 米外，放松眼睛</div>
				<div class="tip">🧘 起立深呼吸几次</div>
				<div class="tip">💧 顺便去喝杯水</div>
			</div>
			<div class="btn-row">
				<button class="btn-secondary" @click="extend">继续工作</button>
				<button class="btn-ghost" @click="skip">结束休息</button>
			</div>
		</div>

		<div v-else-if="isWaterAlert" class="water-content">
			<div class="tips-card">
				<div class="tip">💧 现在喝几口水，放松一下。</div>
				<div class="tip">⌛ 这个提醒会在 5 秒后自动关闭。</div>
			</div>
			<button class="btn-primary" @click="dismissWater">知道了</button>
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
	activeAlert: "sitting",
	sittingRemaining: 0,
	waterRemaining: 80 * 60,
	restRemaining: 5 * 60,
	waterAlertRemaining: 5,
	autoRestSecs: 10,
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
/** @type {number|null} */
let restMelodyTimeout = null;
/** @type {string | null} */
let previousAlertKind = null;

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

const activeAlert = computed(() => state.value.activeAlert);
const isSittingAlert = computed(() => activeAlert.value === "sitting");
const isRestingAlert = computed(() => activeAlert.value === "resting");
const isWaterAlert = computed(() => activeAlert.value === "water");

const circumference = 2 * Math.PI * 70; // ≈ 439.8

const progressRatio = computed(() => {
	const s = state.value;
	if (isRestingAlert.value) {
		return s.restRemaining / (s.restDuration || 5 * 60);
	}
	if (isWaterAlert.value) {
		return s.waterAlertRemaining / 5;
	}
	return Math.max(0, s.sittingRemaining / s.sittingInterval);
});

const dashOffset = computed(() => circumference * (1 - progressRatio.value));

const displayTime = computed(() => {
	const s = state.value;
	if (isRestingAlert.value) return formatTime(s.restRemaining);
	if (isWaterAlert.value) return formatTime(s.waterAlertRemaining);
	if (isSittingAlert.value) return formatTime(autoRestCountdown.value);
	return formatTime(s.sittingRemaining < 0 ? 0 : s.sittingRemaining);
});

const windowTitle = computed(() => {
	if (isRestingAlert.value) return "休息中";
	if (isWaterAlert.value) return "喝水提醒";
	return "休息提醒";
});

const titleText = computed(() => {
	if (isRestingAlert.value) return "休息一下";
	if (isWaterAlert.value) return "该喝水了";
	return "准备开始休息";
});

const ringLabel = computed(() => {
	if (isRestingAlert.value) return "休息剩余";
	if (isWaterAlert.value) return "自动关闭";
	return "后自动休息";
});

const iconClass = computed(() => {
	if (isRestingAlert.value) return "icon--rest";
	if (isWaterAlert.value) return "icon--water";
	return "icon--alert";
});

const iconText = computed(() => {
	if (isRestingAlert.value) return "🌿";
	if (isWaterAlert.value) return "💧";
	return "🪑";
});

// ── 自动开始休息倒计时 ───────────────────

function startAutoCountdown() {
	stopAutoCountdown();
	autoRestCountdown.value = state.value.autoRestSecs || 10;
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

const ALERT_SOUND_NOTES = {
	sitting: [
		{ freq: 659, start: 0, duration: 0.12 },
		{ freq: 784, start: 0.16, duration: 0.12 },
		{ freq: 988, start: 0.34, duration: 0.18 },
	],
	resting: [
		{ freq: 523, start: 0, duration: 0.18 },
		{ freq: 659, start: 0.2, duration: 0.18 },
		{ freq: 784, start: 0.42, duration: 0.22 },
		{ freq: 1047, start: 0.7, duration: 0.3 },
	],
	"resting-loop": [
		{ freq: 523, start: 0, duration: 0.28 },
		{ freq: 587, start: 0.32, duration: 0.24 },
		{ freq: 659, start: 0.62, duration: 0.3 },
		{ freq: 587, start: 0.98, duration: 0.24 },
		{ freq: 523, start: 1.28, duration: 0.34 },
	],
	"resting-end": [
		{ freq: 1047, start: 0, duration: 0.18 },
		{ freq: 784, start: 0.22, duration: 0.18 },
		{ freq: 659, start: 0.46, duration: 0.18 },
		{ freq: 523, start: 0.72, duration: 0.34 },
	],
	water: [
		{ freq: 880, start: 0, duration: 0.09 },
		{ freq: 1174, start: 0.12, duration: 0.12 },
	],
};

/**
 * 使用 Web Audio API 生成简短提示音，避免额外音频素材依赖。
 * @param {"sitting" | "resting" | "resting-loop" | "resting-end" | "water"} kind
 * @returns {void}
 */
function playAlertSound(kind) {
	const AudioContextCtor = window.AudioContext || window.webkitAudioContext;
	if (!AudioContextCtor) return;

	const notes = ALERT_SOUND_NOTES[kind];
	if (!notes?.length) return;

	const ctx = new AudioContextCtor();
	const now = ctx.currentTime;

	for (const note of notes) {
		const oscillator = ctx.createOscillator();
		const gainNode = ctx.createGain();
		oscillator.type = "sine";
		oscillator.frequency.value = note.freq;
		gainNode.gain.setValueAtTime(0.0001, now + note.start);
		const peakGain = kind === "resting-loop" ? 0.07 : 0.12;
		const attack = kind === "resting-loop" ? 0.04 : 0.02;
		gainNode.gain.exponentialRampToValueAtTime(
			peakGain,
			now + note.start + attack,
		);
		gainNode.gain.exponentialRampToValueAtTime(
			0.0001,
			now + note.start + note.duration,
		);
		oscillator.connect(gainNode);
		gainNode.connect(ctx.destination);
		oscillator.start(now + note.start);
		oscillator.stop(now + note.start + note.duration);
	}

	setTimeout(
		() => {
			ctx.close();
		},
		kind === "resting-loop" ? 2200 : 1200,
	);
}

/**
 * 根据提醒状态切换播放对应提示音，避免每秒 tick 重复触发。
 * @param {"sitting" | "resting" | "water" | null} nextAlertKind
 * @returns {void}
 */
function syncAlertSound(nextAlertKind) {
	const previous = previousAlertKind;
	previousAlertKind = nextAlertKind ?? null;

	if (nextAlertKind === previous) return;
	if (previous === "resting" && nextAlertKind !== "resting") {
		playAlertSound("resting-end");
		return;
	}
	if (nextAlertKind) {
		playAlertSound(nextAlertKind);
	}
}

/**
 * 休息阶段按间隔播放短旋律，避免最后一次提示和结束音撞在一起。
 * @returns {void}
 */
function startRestMelodyLoop() {
	stopRestMelodyLoop();
	const scheduleNext = () => {
		restMelodyTimeout = setTimeout(() => {
			restMelodyTimeout = null;
			if (state.value.activeAlert !== "resting") return;
			if (state.value.restRemaining <= 30) return;

			playAlertSound("resting-loop");
			scheduleNext();
		}, 30000);
	};

	// 让尾声提前收口，避免与结束音挤在同一秒。
	scheduleNext();
}

function stopRestMelodyLoop() {
	if (restMelodyTimeout !== null) {
		clearTimeout(restMelodyTimeout);
		restMelodyTimeout = null;
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
	() => activeAlert.value,
	(alertKind) => {
		if (alertKind === "sitting") {
			startAutoCountdown();
		} else {
			stopAutoCountdown();
		}

		if (alertKind === "resting") {
			startRestMelodyLoop();
		} else {
			stopRestMelodyLoop();
		}

		syncAlertSound(alertKind);
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

function dismissWater() {
	invoke("user_action", { action: "dismiss_water" });
}

/** 最小化提醒窗口，便于稍后从任务栏恢复 */
function minimizeWindow() {
	appWindow.minimize();
}

// ── 生命周期 ─────────────────────────────

onMounted(async () => {
	state.value = await invoke("get_timer_state");

	if (state.value.activeAlert === "sitting") {
		startAutoCountdown();
	}
	previousAlertKind = state.value.activeAlert ?? null;

	unlistenTick = await listen("timer-tick", ({ payload }) => {
		state.value = payload;
	});
});

onUnmounted(() => {
	stopAutoCountdown();
	stopRestMelodyLoop();
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

.icon--water {
	background: #dbeafe;
	border: 1.5px solid #93c5fd;
}

/* ── 标题 ── */
.title-area {
	text-align: center;
}

.title {
	font-size: 20px;
	font-weight: 700;
	color: #111827;
	margin: 0;
	letter-spacing: -0.02em;
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

.water-content {
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
