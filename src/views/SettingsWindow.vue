<template>
	<div class="settings-root">
		<aside class="sidebar">
			<div class="sidebar-brand">歇会儿</div>

			<button
				v-for="item in tabs"
				:key="item.key"
				:type="button"
				:class="['sidebar-tab', { 'sidebar-tab--active': activeTab === item.key }]"
				@click="activeTab = item.key"
			>
				<span class="sidebar-tab__icon">{{ item.icon }}</span>
				<span class="sidebar-tab__label">{{ item.label }}</span>
			</button>
		</aside>

		<section class="main-panel">
			<header class="panel-header">
				<h1 class="panel-title">{{ currentTab.label }}</h1>
				<el-tooltip
					content="双击程序后常驻托盘。左键托盘查看时间，右键托盘打开菜单。"
					placement="left"
				>
					<button class="help-button" type="button">?</button>
				</el-tooltip>
			</header>

			<div class="panel-body">
				<template v-if="activeTab === 'basic'">
					<div class="switch-row">
						<span class="row-label">开机自动启动</span>
						<el-switch v-model="form.autostartEnabled" :disabled="!autostartSupported" />
					</div>

					<div class="switch-row">
						<span class="row-label">静默启动</span>
						<el-switch v-model="form.silentStart" />
					</div>

					<div class="path-row">
						<span class="path-label">配置文件</span>
						<el-input :model-value="configPath" readonly class="path-input">
							<template #append>
								<button class="path-button" type="button" @click="openConfigFolder">
									📂
								</button>
							</template>
						</el-input>
					</div>
				</template>

				<template v-else-if="activeTab === 'rest'">
					<div class="switch-row">
						<span class="row-label">启用久坐提醒</span>
						<el-switch v-model="form.restEnabled" />
					</div>

					<div class="slider-list">
						<div class="slider-row">
							<span class="slider-label">提醒间隔</span>
							<div class="slider-control">
								<el-slider
									v-model="form.sittingIntervalMins"
									:min="15"
									:max="120"
									:step="5"
									size="small"
								/>
							</div>
							<div class="number-box">
								<el-input-number
									v-model="form.sittingIntervalMins"
									:min="15"
									:max="120"
									:step="5"
									size="small"
									controls-position="right"
								/>
								<span class="number-unit">分钟</span>
							</div>
						</div>

						<div class="slider-row">
							<span class="slider-label">延长等待</span>
							<div class="slider-control">
								<el-slider
									v-model="form.extendDurationMins"
									:min="1"
									:max="30"
									:step="1"
									size="small"
								/>
							</div>
							<div class="number-box">
								<el-input-number
									v-model="form.extendDurationMins"
									:min="1"
									:max="30"
									:step="1"
									size="small"
									controls-position="right"
								/>
								<span class="number-unit">分钟</span>
							</div>
						</div>

						<div class="slider-row">
							<span class="slider-label">休息时长</span>
							<div class="slider-control">
								<el-slider
									v-model="form.restDurationMins"
									:min="1"
									:max="30"
									:step="1"
									size="small"
								/>
							</div>
							<div class="number-box">
								<el-input-number
									v-model="form.restDurationMins"
									:min="1"
									:max="30"
									:step="1"
									size="small"
									controls-position="right"
								/>
								<span class="number-unit">分钟</span>
							</div>
						</div>

						<div class="slider-row">
							<span class="slider-label">自动开始</span>
							<div class="slider-control">
								<el-slider
									v-model="form.autoRestSecs"
									:min="5"
									:max="60"
									:step="5"
									size="small"
								/>
							</div>
							<div class="number-box">
								<el-input-number
									v-model="form.autoRestSecs"
									:min="5"
									:max="60"
									:step="5"
									size="small"
									controls-position="right"
								/>
								<span class="number-unit">秒</span>
							</div>
						</div>
					</div>
				</template>

				<template v-else>
					<div class="switch-row">
						<span class="row-label">启用喝水提醒</span>
						<el-switch v-model="form.waterEnabled" />
					</div>

					<div class="slider-list">
						<div class="slider-row">
							<span class="slider-label">提醒间隔</span>
							<div class="slider-control">
								<el-slider
									v-model="form.waterIntervalMins"
									:min="30"
									:max="180"
									:step="5"
									size="small"
								/>
							</div>
							<div class="number-box">
								<el-input-number
									v-model="form.waterIntervalMins"
									:min="30"
									:max="180"
									:step="5"
									size="small"
									controls-position="right"
								/>
								<span class="number-unit">分钟</span>
							</div>
						</div>
					</div>
				</template>
			</div>

			<footer class="panel-footer">
				<button class="secondary-button" type="button" @click="resetDefaults">恢复默认</button>
				<button class="primary-button" :class="{ 'primary-button--saved': justSaved }" type="button" @click="save">
					{{ justSaved ? "已保存" : "保存" }}
				</button>
			</footer>
		</section>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { computed, onMounted, reactive, ref } from "vue";

const tabs = [
	{ key: "basic", icon: "⚙️", label: "基础设置" },
	{ key: "rest", icon: "🪑", label: "休息提醒" },
	{ key: "water", icon: "💧", label: "喝水提醒" },
];

/** 默认配置 */
const DEFAULTS = {
	silentStart: false,
	autostartEnabled: false,
	restEnabled: true,
	waterEnabled: true,
	sittingIntervalMins: 50,
	waterIntervalMins: 80,
	restDurationMins: 5,
	extendDurationMins: 5,
	autoRestSecs: 10,
};

const activeTab = ref("basic");
const form = reactive({ ...DEFAULTS });
const justSaved = ref(false);
const configPath = ref("");
const autostartSupported = ref(false);

const currentTab = computed(
	() => tabs.find((item) => item.key === activeTab.value) ?? tabs[0],
);

/** 从后端配置文件加载设置 */
async function loadSettings() {
	const payload = await invoke("get_settings");
	configPath.value = payload.configPath;
	autostartSupported.value = payload.autostartSupported;

	Object.assign(form, {
		silentStart: payload.config.silentStart,
		autostartEnabled: payload.config.autostartEnabled,
		restEnabled: payload.config.restEnabled,
		waterEnabled: payload.config.waterEnabled,
		sittingIntervalMins: payload.config.sittingIntervalSecs / 60,
		waterIntervalMins: payload.config.waterIntervalSecs / 60,
		restDurationMins: payload.config.restDurationSecs / 60,
		extendDurationMins: payload.config.extendDurationSecs / 60,
		autoRestSecs: payload.config.autoRestSecs,
	});
}

/** 打开配置文件所在位置 */
async function openConfigFolder() {
	if (!configPath.value) return;
	await revealItemInDir(configPath.value);
}

/** 持久化到本地 data/config.json 并同步到 Rust */
async function save() {
	await invoke("set_timer_config", {
		silentStart: form.silentStart,
		autoRestSecs: form.autoRestSecs,
		sittingIntervalSecs: form.sittingIntervalMins * 60,
		waterIntervalSecs: form.waterIntervalMins * 60,
		restDurationSecs: form.restDurationMins * 60,
		extendDurationSecs: form.extendDurationMins * 60,
		restEnabled: form.restEnabled,
		waterEnabled: form.waterEnabled,
		autostartEnabled: form.autostartEnabled,
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
	loadSettings();
});
</script>

<style scoped>
.settings-root {
	width: 100%;
	height: 100%;
	display: flex;
	background: #f3f5f8;
	color: #111827;
	font-family: "Segoe UI Variable", "Segoe UI", "PingFang SC", "Microsoft YaHei",
		sans-serif;
	overflow: hidden;
}

.sidebar {
	width: 180px;
	flex-shrink: 0;
	padding: 14px 10px;
	background: #ffffff;
	border-right: 1px solid #e5e7eb;
	display: flex;
	flex-direction: column;
	gap: 6px;
}

.sidebar-brand {
	height: 34px;
	display: flex;
	align-items: center;
	padding: 0 10px;
	margin-bottom: 8px;
	font-size: 18px;
	font-weight: 700;
	letter-spacing: -0.02em;
}

.sidebar-tab {
	height: 36px;
	border: none;
	border-radius: 10px;
	background: transparent;
	padding: 0 10px;
	display: flex;
	align-items: center;
	gap: 8px;
	cursor: pointer;
	font-size: 13px;
	font-weight: 600;
	color: #4b5563;
	text-align: left;
}

.sidebar-tab:hover {
	background: #f3f4f6;
}

.sidebar-tab--active {
	background: #e8f0fe;
	color: #1d4ed8;
}

.sidebar-tab__icon {
	width: 18px;
	text-align: center;
	flex-shrink: 0;
}

.main-panel {
	flex: 1;
	padding: 14px 14px 12px;
	display: flex;
	flex-direction: column;
	gap: 10px;
	overflow: hidden;
}

.panel-header {
	height: 48px;
	flex-shrink: 0;
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 0 4px 0 8px;
}

.panel-title {
	margin: 0;
	font-size: 18px;
	font-weight: 700;
	letter-spacing: -0.02em;
}

.help-button {
	width: 28px;
	height: 28px;
	border: none;
	border-radius: 999px;
	background: #ffffff;
	color: #6b7280;
	font-size: 13px;
	font-weight: 700;
	cursor: pointer;
	box-shadow: inset 0 0 0 1px #e5e7eb;
}

.panel-body {
	flex: 1;
	background: #ffffff;
	border: 1px solid #e5e7eb;
	border-radius: 14px;
	padding: 12px 16px;
	display: flex;
	flex-direction: column;
	gap: 10px;
	overflow: hidden;
}

.switch-row,
.path-row,
.slider-row {
	min-height: 38px;
	display: flex;
	align-items: center;
	gap: 12px;
}

.row-label,
.path-label,
.slider-label {
	width: 96px;
	flex-shrink: 0;
	font-size: 13px;
	font-weight: 600;
	color: #374151;
}

.path-row {
	margin-top: auto;
}

.path-input {
	flex: 1;
	min-width: 0;
}

.path-button {
	width: 32px;
	height: 30px;
	border: none;
	background: transparent;
	cursor: pointer;
	font-size: 16px;
	line-height: 1;
}

.slider-list {
	display: flex;
	flex-direction: column;
	gap: 8px;
}

.slider-control {
	flex: 1;
	min-width: 0;
	padding-right: 4px;
}

.number-box {
	width: 124px;
	flex-shrink: 0;
	display: flex;
	align-items: center;
	justify-content: flex-end;
	gap: 6px;
	white-space: nowrap;
}

.number-unit {
	font-size: 12px;
	color: #6b7280;
}

.panel-footer {
	height: 36px;
	flex-shrink: 0;
	display: flex;
	justify-content: flex-end;
	gap: 8px;
}

.secondary-button,
.primary-button {
	height: 36px;
	padding: 0 16px;
	border-radius: 10px;
	font-size: 13px;
	font-weight: 600;
	cursor: pointer;
}

.secondary-button {
	border: 1px solid #d1d5db;
	background: #ffffff;
	color: #4b5563;
}

.primary-button {
	border: none;
	background: #2563eb;
	color: #ffffff;
	min-width: 88px;
}

.primary-button--saved {
	background: #16a34a;
}

:deep(.el-switch) {
	--el-switch-on-color: #2563eb;
	--el-switch-off-color: #cbd5e1;
}

:deep(.el-input__wrapper) {
	box-shadow: inset 0 0 0 1px #d1d5db;
}

:deep(.el-input-group__append) {
	padding: 0;
	background: #f9fafb;
}

:deep(.el-slider) {
	width: 100%;
}

:deep(.el-slider__runway) {
	margin: 0;
}

:deep(.el-input-number) {
	width: 82px;
}
</style>
