<template>
	<!-- mousedown 在非按钮区域时触发系统拖动 -->
	<div class="popup-root anim-fade-in" @mousedown="handleMouseDown">
		<!-- 顶栏：状态徽章 + 操作按钮 -->
		<div class="header-row">
			<span :class="['badge', phaseBadgeClass]">
				<span class="dot" />
				{{ phaseLabel }}
			</span>
			<div class="header-actions">
				<button :class="['icon-btn', { 'icon-btn--active': pinned }]" :title="pinned ? '取消固定' : '固定窗口'"
					@click.stop="togglePin">
					📌
				</button>
				<button class="icon-btn" title="关闭" @click.stop="closePopup">✕</button>
			</div>
		</div>

		<!-- 久坐倒计时 -->
		<div class="timer-block">
			<div :class="['timer-label', { 'timer-label--muted': state.restTimerPaused }]">
				<span class="timer-icon">{{ timerLabelIcon }}</span>
				<span class="timer-label-text">{{ timerLabel }}</span>
			</div>
			<div :class="['countdown', 'size-main', { 'countdown--muted': state.restTimerPaused }]">
				{{ mainCountdown }}
			</div>
			<div class="action-row">
				<button class="icon-action-btn" title="重置休息计时" @click.stop="resetRestTimer">
					↻
				</button>
				<button :class="['icon-action-btn', { 'icon-action-btn--active': state.restTimerPaused }]"
					:title="state.restTimerPaused ? '恢复休息计时' : '暂停休息计时'" @click.stop="toggleRestPause">
					{{ state.restTimerPaused ? "▶" : "⏸" }}
				</button>
			</div>
		</div>

		<!-- 分隔线 -->
		<div class="divider" />

		<!-- 喝水倒计时 -->
		<div class="water-row">
			<span class="water-icon">💧</span>
			<span :class="['water-text', { 'water-text--muted': state.waterTimerPaused }]">喝水</span>
			<span :class="['water-time', { 'water-time--muted': state.waterTimerPaused }]">
				{{ formatTime(state.waterRemaining) }}
			</span>
			<button class="icon-action-btn icon-action-btn--compact" title="重置喝水计时" @click.stop="resetWaterTimer">
				↻
			</button>
			<button
				:class="['icon-action-btn', 'icon-action-btn--compact', { 'icon-action-btn--active': state.waterTimerPaused }]"
				:title="state.waterTimerPaused ? '恢复喝水计时' : '暂停喝水计时'" @click.stop="toggleWaterPause">
				{{ state.waterTimerPaused ? "▶" : "⏸" }}
			</button>
		</div>

		<div class="helper-text">右键托盘图标可打开设置页</div>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { computed, onMounted, onUnmounted, ref } from "vue";

const appWindow = getCurrentWebviewWindow();
const PHASE_META = {
	running: {
		label: "专注中",
		badgeClass: "badge-accent",
		timerLabel: "距离下次休息",
		timerIcon: "⏱️",
	},
	triggered: {
		label: "该休息了",
		badgeClass: "badge-warning",
		timerLabel: "距离下次休息",
		timerIcon: "⏱️",
	},
	resting: {
		label: "休息中",
		badgeClass: "badge-success",
		timerLabel: "休息剩余时间",
		timerIcon: "🛌",
	},
};

const state = ref({
	phase: "running",
	restTimerPaused: false,
	waterTimerPaused: false,
	sittingRemaining: 50 * 60,
	waterRemaining: 80 * 60,
	restRemaining: 0,
	sittingInterval: 50 * 60,
	waterInterval: 80 * 60,
});

/** 是否已固定（固定后 blur 不自动隐藏） */
const pinned = ref(true);
const dragging = ref(false);

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

const mainCountdown = computed(() => {
	const s = state.value;
	if (s.phase === "resting") return formatTime(s.restRemaining);
	return formatTime(s.sittingRemaining);
});

const phaseMeta = computed(
	() => PHASE_META[state.value.phase] ?? PHASE_META.running,
);

const phaseLabel = computed(() => phaseMeta.value.label);

const phaseBadgeClass = computed(() => phaseMeta.value.badgeClass);

const timerLabel = computed(() => phaseMeta.value.timerLabel);

const timerLabelIcon = computed(() => phaseMeta.value.timerIcon);

// ── 操作 ─────────────────────────────────

function closePopup() {
	invoke("hide_popup");
}

function togglePin() {
	pinned.value = !pinned.value;
}

function toggleRestPause() {
	invoke("user_action", { action: "toggle_rest_pause" });
}

function resetRestTimer() {
	invoke("user_action", { action: "reset_rest_timer" });
}

function toggleWaterPause() {
	invoke("user_action", { action: "toggle_water_pause" });
}

function resetWaterTimer() {
	invoke("user_action", { action: "reset_water_timer" });
}

/**
 * 点击非按钮区域时触发系统级窗口拖动
 * @param {MouseEvent} e
 */
function handleMouseDown(e) {
	if (e.target.closest("button")) return;
	dragging.value = true;
	setTimeout(() => {
		dragging.value = false;
	}, 400);
	appWindow.startDragging();
}

// ── 生命周期 ─────────────────────────────

let unlistenTick = null;
let unlistenBlur = null;

onMounted(async () => {
	state.value = await invoke("get_timer_state");

	unlistenTick = await listen("timer-tick", ({ payload }) => {
		state.value = payload;
	});

	unlistenBlur = await appWindow.listen("tauri://blur", () => {
		if (!pinned.value && !dragging.value) {
			invoke("hide_popup");
		}
	});
});

onUnmounted(() => {
	unlistenTick?.();
	unlistenBlur?.();
});
</script>

<style scoped>
.popup-root {
	width: 240px;
	height: 210px;
	border-radius: 14px;
	padding: 10px 14px 10px;
	display: flex;
	flex-direction: column;
	gap: 6px;
	position: relative;
	overflow: hidden;
	background: #ffffff;
	border: 1px solid #e2e5ea;
	box-shadow:
		0 4px 16px rgba(0, 0, 0, 0.12),
		0 1px 4px rgba(0, 0, 0, 0.06);
	cursor: move;
}

/* ── 顶栏 ── */
.header-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	flex-shrink: 0;
}

.header-actions {
	display: flex;
	align-items: center;
	gap: 2px;
}

.badge {
	display: inline-flex;
	align-items: center;
	gap: 4px;
	padding: 2px 8px;
	border-radius: 100px;
	font-size: 11px;
	font-weight: 500;
	letter-spacing: 0.02em;
}

.badge-accent {
	background: #ede9fe;
	color: #6d28d9;
}

.badge-warning {
	background: #fff7ed;
	color: #c2410c;
}

.badge-success {
	background: #f0fdf4;
	color: #166534;
}

.badge-muted {
	background: #f3f4f6;
	color: #6b7280;
}

.dot {
	display: inline-block;
	width: 5px;
	height: 5px;
	border-radius: 50%;
	background: currentColor;
}

.icon-btn {
	background: none;
	border: none;
	cursor: pointer;
	font-size: 11px;
	color: #9ca3af;
	padding: 3px 4px;
	border-radius: 5px;
	line-height: 1;
	transition:
		background 0.15s,
		color 0.15s;
}

.icon-btn:hover {
	background: #f3f4f6;
	color: #374151;
}

.icon-btn--active {
	color: #7c3aed;
	background: #ede9fe;
}

/* ── 久坐计时区 ── */
.timer-block {
	flex: 1;
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 6px;
}

.timer-label {
	display: flex;
	align-items: center;
	gap: 4px;
	margin-bottom: 2px;
}

.timer-icon {
	font-size: 13px;
}

.timer-label-text {
	font-size: 11px;
	color: #9ca3af;
}

.timer-label--muted .timer-icon,
.timer-label--muted .timer-label-text {
	color: #9ca3af;
}

/* 覆盖全局 countdown 的暗色渐变为浅色文字 */
.countdown {
	font-family: "JetBrains Mono", "Fira Code", monospace;
	font-weight: 600;
	line-height: 1;
	letter-spacing: -0.02em;
	background: none;
	-webkit-text-fill-color: #111827;
	color: #111827;
}

.countdown.size-main {
	font-size: 38px;
}

.countdown--muted {
	-webkit-text-fill-color: #9ca3af;
	color: #9ca3af;
}

.action-row {
	display: flex;
	gap: 6px;
}


.icon-action-btn {
	width: 24px;
	height: 24px;
	border: none;
	background: transparent;
	color: #6b7280;
	border-radius: 8px;
	padding: 0;
	font-size: 13px;
	line-height: 1;
	cursor: pointer;
	transition:
		background 0.15s,
		color 0.15s,
		opacity 0.15s;
	opacity: 0.82;
}


.icon-action-btn:hover {
	background: #f3f4f6;
	color: #111827;
	opacity: 1;
}


.icon-action-btn--active {
	color: #6d28d9;
}


.icon-action-btn--active:hover {
	background: #ede9fe;
}

.icon-action-btn--compact {
	flex-shrink: 0;
}

/* ── 分隔线 ── */
.divider {
	height: 1px;
	background: #f0f2f5;
	flex-shrink: 0;
}

/* ── 喝水行 ── */
.water-row {
	display: flex;
	align-items: center;
	gap: 5px;
	flex-shrink: 0;
	flex-wrap: wrap;
}

.water-icon {
	font-size: 13px;
}

.water-text {
	font-size: 11px;
	color: #9ca3af;
	flex: 1;
}

.water-text--muted {
	color: #c0c4cc;
}

.water-time {
	font-family: "JetBrains Mono", monospace;
	font-size: 12px;
	color: #374151;
	font-weight: 500;
	margin-left: auto;
}

.water-time--muted {
	color: #9ca3af;
}

.helper-text {
	font-size: 10px;
	color: #9ca3af;
	text-align: center;
	flex-shrink: 0;
}
</style>
