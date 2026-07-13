<template>
	<div class="settings-root">
		<div class="settings-header">
			<h1 class="settings-title">设置</h1>
			<p class="settings-sub">歇会儿 · 定时提醒</p>
		</div>

		<div class="settings-body">
			<!-- 久坐提醒 -->
			<section class="section">
				<div class="section-head">
					<span class="section-icon">🪑</span>
					<div>
						<div class="section-title">久坐提醒</div>
						<div class="section-desc">连续专注超过设定时间后弹窗提醒</div>
					</div>
				</div>
				<div class="field-group">
					<label class="field">
						<span class="field-label">提醒间隔</span>
						<div class="slider-wrap">
							<input
								v-model.number="form.sittingIntervalMins"
								type="range"
								min="15"
								max="120"
								step="5"
								class="slider"
							/>
							<span class="slider-value">{{ form.sittingIntervalMins }} 分钟</span>
						</div>
					</label>
					<label class="field">
						<span class="field-label">延长等待</span>
						<div class="slider-wrap">
							<input
								v-model.number="form.extendDurationMins"
								type="range"
								min="1"
								max="30"
								step="1"
								class="slider"
							/>
							<span class="slider-value">{{ form.extendDurationMins }} 分钟</span>
						</div>
					</label>
				</div>
			</section>

			<!-- 休息设置 -->
			<section class="section">
				<div class="section-head">
					<span class="section-icon">🌿</span>
					<div>
						<div class="section-title">休息设置</div>
						<div class="section-desc">开始休息后的倒计时时长</div>
					</div>
				</div>
				<div class="field-group">
					<label class="field">
						<span class="field-label">休息时长</span>
						<div class="slider-wrap">
							<input
								v-model.number="form.restDurationMins"
								type="range"
								min="1"
								max="30"
								step="1"
								class="slider"
							/>
							<span class="slider-value">{{ form.restDurationMins }} 分钟</span>
						</div>
					</label>
					<label class="field">
						<span class="field-label">自动开始（秒）</span>
						<div class="slider-wrap">
							<input
								v-model.number="form.autoRestSecs"
								type="range"
								min="5"
								max="60"
								step="5"
								class="slider"
							/>
							<span class="slider-value">{{ form.autoRestSecs }} 秒</span>
						</div>
					</label>
				</div>
			</section>

			<!-- 喝水提醒 -->
			<section class="section">
				<div class="section-head">
					<span class="section-icon">💧</span>
					<div>
						<div class="section-title">喝水提醒</div>
						<div class="section-desc">系统通知弱提醒，不打断工作</div>
					</div>
				</div>
				<div class="field-group">
					<label class="field">
						<span class="field-label">提醒间隔</span>
						<div class="slider-wrap">
							<input
								v-model.number="form.waterIntervalMins"
								type="range"
								min="30"
								max="180"
								step="5"
								class="slider"
							/>
							<span class="slider-value">{{ form.waterIntervalMins }} 分钟</span>
						</div>
					</label>
				</div>
			</section>
		</div>

		<!-- 底部按钮 -->
		<div class="settings-footer">
			<button class="btn-reset" @click="resetDefaults">恢复默认</button>
			<button class="btn-save" :class="{ saved: justSaved }" @click="save">
				{{ justSaved ? "✓ 已保存" : "保存" }}
			</button>
		</div>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { onMounted, reactive, ref } from "vue";

/** 默认配置 */
const DEFAULTS = {
	sittingIntervalMins: 50,
	waterIntervalMins: 80,
	restDurationMins: 5,
	extendDurationMins: 5,
	autoRestSecs: 10,
};

const form = reactive({ ...DEFAULTS });
const justSaved = ref(false);

/** 从 localStorage 加载持久化配置 */
function loadFromStorage() {
	try {
		const raw = localStorage.getItem("tab-settings");
		if (raw) {
			const saved = JSON.parse(raw);
			Object.assign(form, saved);
		}
	} catch {
		// 解析失败则使用默认值
	}
}

/** 持久化到 localStorage 并同步到 Rust 计时器 */
async function save() {
	localStorage.setItem("tab-settings", JSON.stringify({ ...form }));

	await invoke("set_timer_config", {
		sittingIntervalSecs: form.sittingIntervalMins * 60,
		waterIntervalSecs: form.waterIntervalMins * 60,
		restDurationSecs: form.restDurationMins * 60,
		extendDurationSecs: form.extendDurationMins * 60,
	});

	justSaved.value = true;
	setTimeout(() => {
		justSaved.value = false;
	}, 2000);
}

function resetDefaults() {
	Object.assign(form, DEFAULTS);
}

onMounted(() => {
	loadFromStorage();
});
</script>

<style scoped>
.settings-root {
	width: 100%;
	height: 100%;
	display: flex;
	flex-direction: column;
	background: #f8fafc;
	font-family: "Inter", "PingFang SC", "Microsoft YaHei", sans-serif;
	overflow: hidden;
}

/* ── 头部 ── */
.settings-header {
	padding: 20px 24px 0;
}

.settings-title {
	font-size: 20px;
	font-weight: 700;
	color: #111827;
	margin: 0 0 2px;
	letter-spacing: -0.02em;
}

.settings-sub {
	font-size: 12px;
	color: #9ca3af;
	margin: 0;
}

/* ── 主体 ── */
.settings-body {
	flex: 1;
	overflow-y: auto;
	padding: 16px 24px;
	display: flex;
	flex-direction: column;
	gap: 12px;
}

/* ── 分组卡片 ── */
.section {
	background: #ffffff;
	border: 1px solid #e9eaf0;
	border-radius: 14px;
	padding: 16px;
}

.section-head {
	display: flex;
	align-items: flex-start;
	gap: 10px;
	margin-bottom: 14px;
}

.section-icon {
	font-size: 20px;
	line-height: 1;
	flex-shrink: 0;
	margin-top: 1px;
}

.section-title {
	font-size: 14px;
	font-weight: 600;
	color: #1f2937;
	margin-bottom: 2px;
}

.section-desc {
	font-size: 12px;
	color: #6b7280;
	line-height: 1.4;
}

/* ── 字段组 ── */
.field-group {
	display: flex;
	flex-direction: column;
	gap: 12px;
}

.field {
	display: flex;
	flex-direction: column;
	gap: 6px;
	cursor: default;
}

.field-label {
	font-size: 12px;
	font-weight: 500;
	color: #374151;
}

/* ── 滑块 ── */
.slider-wrap {
	display: flex;
	align-items: center;
	gap: 10px;
}

.slider {
	flex: 1;
	height: 4px;
	border-radius: 2px;
	appearance: none;
	background: linear-gradient(
		to right,
		#7c3aed 0%,
		#7c3aed calc(var(--ratio, 0) * 100%),
		#e5e7eb calc(var(--ratio, 0) * 100%),
		#e5e7eb 100%
	);
	outline: none;
	cursor: pointer;
}

.slider::-webkit-slider-thumb {
	appearance: none;
	width: 16px;
	height: 16px;
	border-radius: 50%;
	background: #7c3aed;
	box-shadow: 0 1px 4px rgba(124, 58, 237, 0.4);
	cursor: pointer;
	transition: transform 0.15s;
}

.slider::-webkit-slider-thumb:hover {
	transform: scale(1.15);
}

.slider-value {
	font-size: 12px;
	font-weight: 600;
	color: #7c3aed;
	min-width: 58px;
	text-align: right;
}

/* ── 底部按钮 ── */
.settings-footer {
	display: flex;
	justify-content: flex-end;
	align-items: center;
	gap: 10px;
	padding: 14px 24px;
	border-top: 1px solid #e9eaf0;
	background: #ffffff;
	flex-shrink: 0;
}

.btn-reset {
	padding: 8px 16px;
	background: none;
	border: 1px solid #e5e7eb;
	border-radius: 8px;
	font-size: 13px;
	font-weight: 500;
	color: #6b7280;
	cursor: pointer;
	font-family: inherit;
	transition:
		background 0.15s,
		color 0.15s;
}

.btn-reset:hover {
	background: #f3f4f6;
	color: #374151;
}

.btn-save {
	padding: 8px 24px;
	background: linear-gradient(135deg, #7c3aed, #a78bfa);
	color: #fff;
	border: none;
	border-radius: 8px;
	font-size: 13px;
	font-weight: 600;
	cursor: pointer;
	font-family: inherit;
	transition:
		transform 0.15s,
		box-shadow 0.15s,
		background 0.2s;
	box-shadow: 0 3px 10px rgba(124, 58, 237, 0.3);
}

.btn-save:hover {
	transform: translateY(-1px);
	box-shadow: 0 5px 16px rgba(124, 58, 237, 0.4);
}

.btn-save.saved {
	background: linear-gradient(135deg, #16a34a, #4ade80);
	box-shadow: 0 3px 10px rgba(22, 163, 74, 0.3);
}
</style>
