<template>
	<div class="popup-root anim-fade-in" data-tauri-drag-region>
		<!-- 顶栏：状态徽章 + 操作按钮 -->
		<div class="header-row">
			<span :class="['badge', phaseBadgeClass]">
				<span class="dot" />
				{{ phaseLabel }}
			</span>
			<div class="header-actions">
				<!-- 置顶 pin：固定后不再 blur 隐藏 -->
				<button
					:class="['icon-btn', { 'icon-btn--active': pinned }]"
					:title="pinned ? '取消固定' : '固定窗口'"
					@click.stop="togglePin"
				>
					📌
				</button>
				<button class="icon-btn" title="关闭" @click.stop="closePopup">✕</button>
			</div>
		</div>

		<!-- 久坐倒计时 -->
		<div class="timer-block">
			<div class="timer-label">
				<span class="timer-icon">🪑</span>
				<span class="timer-label-text">久坐</span>
			</div>
			<div class="countdown size-main">{{ mainCountdown }}</div>
			<div class="timer-sub">{{ subLabel }}</div>
		</div>

		<!-- 分隔线 -->
		<div class="divider" />

		<!-- 喝水倒计时 -->
		<div class="water-row">
			<span class="water-icon">💧</span>
			<span class="water-text">喝水</span>
			<span class="water-time">{{ formatTime(state.waterRemaining) }}</span>
		</div>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { computed, onMounted, onUnmounted, ref } from "vue";

const appWindow = getCurrentWebviewWindow();

const state = ref({
	phase: "running",
	sittingRemaining: 50 * 60,
	waterRemaining: 80 * 60,
	restRemaining: 0,
	sittingInterval: 50 * 60,
	waterInterval: 80 * 60,
});

/** 是否已固定（固定后 blur 不自动隐藏） */
const pinned = ref(false);

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

const phaseLabel = computed(() => {
	switch (state.value.phase) {
		case "running":
			return "专注中";
		case "triggered":
			return "该休息了";
		case "resting":
			return "休息中";
		case "paused":
			return "已暂停";
		default:
			return "—";
	}
});

const phaseBadgeClass = computed(() => {
	switch (state.value.phase) {
		case "running":
			return "badge-accent";
		case "triggered":
			return "badge-warning";
		case "resting":
			return "badge-success";
		case "paused":
			return "badge-muted";
		default:
			return "badge-accent";
	}
});

const subLabel = computed(() => {
	switch (state.value.phase) {
		case "running":
			return "距下次提醒";
		case "triggered":
			return "请处理提醒";
		case "resting":
			return "休息倒计时";
		case "paused":
			return "提醒已暂停";
		default:
			return "";
	}
});

// ── 操作 ─────────────────────────────────

function closePopup() {
	invoke("hide_popup");
}

function togglePin() {
	pinned.value = !pinned.value;
}

// ── 生命周期 ─────────────────────────────

let unlistenTick = null;
let unlistenBlur = null;

onMounted(async () => {
	state.value = await invoke("get_timer_state");

	unlistenTick = await listen("timer-tick", ({ payload }) => {
		state.value = payload;
	});

	// 未固定时，失焦自动隐藏
	unlistenBlur = await appWindow.listen("tauri://blur", () => {
		if (!pinned.value) {
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
	width: 220px;
	height: 180px;
	border-radius: 14px;
	padding: 10px 14px 10px;
	display: flex;
	flex-direction: column;
	gap: 6px;
	position: relative;
	overflow: hidden;
	/* 不透明浅色背景 */
	background: #ffffff;
	border: 1px solid #e2e5ea;
	box-shadow:
		0 4px 16px rgba(0, 0, 0, 0.12),
		0 1px 4px rgba(0, 0, 0, 0.06);
	/* 拖动光标（整个窗口可拖动） */
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

/* 徽章 */
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

/* 图标按钮（置顶 / 关闭） */
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

/* 置顶激活状态 */
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
	gap: 1px;
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

/* 覆盖全局 countdown 的暗色渐变，改为浅色深色文字 */
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

.timer-sub {
	font-size: 10px;
	color: #9ca3af;
	letter-spacing: 0.04em;
	margin-top: 2px;
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
}

.water-icon {
	font-size: 13px;
}

.water-text {
	font-size: 11px;
	color: #9ca3af;
	flex: 1;
}

.water-time {
	font-family: "JetBrains Mono", monospace;
	font-size: 12px;
	color: #374151;
	font-weight: 500;
}
</style>
