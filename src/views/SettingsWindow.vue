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
			<div
				:class="[
					'panel-body',
					activeTab === 'system' ? 'panel-body--system' : 'panel-body--reminder',
				]"
			>
				<template v-if="activeTab === 'system'">
					<div class="content-stack">
						<div class="setting-row">
							<div class="setting-copy">
								<div class="setting-title">开机自动启动</div>
								<div class="setting-desc">随系统启动程序</div>
							</div>
							<div class="setting-control setting-control--switch">
								<el-switch
									v-model="form.autostartEnabled"
									:disabled="!autostartSupported"
								/>
							</div>
						</div>

						<div class="setting-row">
							<div class="setting-copy">
								<div class="setting-title">静默启动</div>
								<div class="setting-desc">启动时直接常驻托盘</div>
							</div>
							<div class="setting-control setting-control--switch">
								<el-switch v-model="form.silentStart" />
							</div>
						</div>

						<div class="setting-row">
							<div class="setting-copy">
								<div class="setting-title">配置文件</div>
								<div class="setting-desc">配置保存在程序目录下的 data 文件夹</div>
							</div>
							<div class="setting-control setting-control--path">
								<el-input :model-value="configPath" readonly class="path-input">
									<template #append>
										<button
											class="path-button"
											type="button"
											@click.stop.prevent="openConfigFolder"
										>
											📂
										</button>
									</template>
								</el-input>
							</div>
						</div>
					</div>
				</template>

				<template v-else>
					<div class="reminder-sections">
						<section class="section-block">
							<h2 class="section-title">久坐提醒</h2>

							<div class="setting-row">
								<div class="setting-copy">
									<div class="setting-title">启用此提醒</div>
								</div>
								<div class="setting-control setting-control--switch">
									<el-switch v-model="form.restEnabled" />
								</div>
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
						</section>

						<div class="section-divider" />

						<section class="section-block">
							<h2 class="section-title">喝水提醒</h2>

							<div class="setting-row">
								<div class="setting-copy">
									<div class="setting-title">启用此提醒</div>
								</div>
								<div class="setting-control setting-control--switch">
									<el-switch v-model="form.waterEnabled" />
								</div>
							</div>

							<div class="slider-list">
								<div class="slider-row">
									<span class="slider-label">提醒间隔</span>
									<div class="slider-control">
										<el-slider
											v-model="form.waterIntervalMins"
											:min="15"
											:max="120"
											:step="5"
											size="small"
										/>
									</div>
									<div class="number-box">
										<el-input-number
											v-model="form.waterIntervalMins"
											:min="15"
											:max="120"
											:step="5"
											size="small"
											controls-position="right"
										/>
										<span class="number-unit">分钟</span>
									</div>
								</div>
							</div>
						</section>
					</div>
				</template>
			</div>

			<footer class="panel-footer">
				<div class="guide-inline">
					💡 提示：本程序关闭后将隐藏至系统托盘。右键托盘图标可重新打开设置页。
				</div>
				<div class="footer-actions">
					<button class="secondary-button" type="button" @click="resetDefaults">
						恢复默认
					</button>
				</div>
			</footer>
		</section>
	</div>
</template>

<script setup>
import { invoke } from "@tauri-apps/api/core";
import { nextTick, onMounted, reactive, ref, watch } from "vue";

const tabs = [
	{ key: "system", icon: "⚙️", label: "系统设置" },
	{ key: "reminder", icon: "⏱️", label: "提醒设置" },
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

const activeTab = ref("system");
const form = reactive({ ...DEFAULTS });
const configPath = ref("");
const autostartSupported = ref(false);
const isHydrating = ref(true);
let lastSavedForm = createFormSnapshot();
let saveQueue = Promise.resolve();

/** 生成当前表单快照，用于比较哪些字段真的发生了变化 */
function createFormSnapshot() {
	return {
		silentStart: form.silentStart,
		autostartEnabled: form.autostartEnabled,
		restEnabled: form.restEnabled,
		waterEnabled: form.waterEnabled,
		sittingIntervalMins: form.sittingIntervalMins,
		waterIntervalMins: form.waterIntervalMins,
		restDurationMins: form.restDurationMins,
		extendDurationMins: form.extendDurationMins,
		autoRestSecs: form.autoRestSecs,
	};
}

/** 只提交有变化的配置，避免无关设置触发额外的注册表写入 */
function buildChangedPayload(nextSnapshot) {
	const payload = {};

	if (nextSnapshot.silentStart !== lastSavedForm.silentStart) {
		payload.silentStart = nextSnapshot.silentStart;
	}
	if (nextSnapshot.autostartEnabled !== lastSavedForm.autostartEnabled) {
		payload.autostartEnabled = nextSnapshot.autostartEnabled;
	}
	if (nextSnapshot.restEnabled !== lastSavedForm.restEnabled) {
		payload.restEnabled = nextSnapshot.restEnabled;
	}
	if (nextSnapshot.waterEnabled !== lastSavedForm.waterEnabled) {
		payload.waterEnabled = nextSnapshot.waterEnabled;
	}
	if (nextSnapshot.sittingIntervalMins !== lastSavedForm.sittingIntervalMins) {
		payload.sittingIntervalSecs = nextSnapshot.sittingIntervalMins * 60;
	}
	if (nextSnapshot.waterIntervalMins !== lastSavedForm.waterIntervalMins) {
		payload.waterIntervalSecs = nextSnapshot.waterIntervalMins * 60;
	}
	if (nextSnapshot.restDurationMins !== lastSavedForm.restDurationMins) {
		payload.restDurationSecs = nextSnapshot.restDurationMins * 60;
	}
	if (nextSnapshot.extendDurationMins !== lastSavedForm.extendDurationMins) {
		payload.extendDurationSecs = nextSnapshot.extendDurationMins * 60;
	}
	if (nextSnapshot.autoRestSecs !== lastSavedForm.autoRestSecs) {
		payload.autoRestSecs = nextSnapshot.autoRestSecs;
	}

	return payload;
}

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

	await nextTick();
	lastSavedForm = createFormSnapshot();
	isHydrating.value = false;
}

/** 打开配置文件所在位置 */
async function openConfigFolder() {
	if (!configPath.value) return;
	await invoke("open_config_dir");
}

/** 持久化到本地 data/config.json 并同步到 Rust */
async function save(payload) {
	await invoke("set_timer_config", payload);
}

function resetDefaults() {
	Object.assign(form, DEFAULTS);
}

watch(
	form,
	async () => {
		if (isHydrating.value) return;
		const nextSnapshot = createFormSnapshot();
		const payload = buildChangedPayload(nextSnapshot);
		if (Object.keys(payload).length === 0) return;

		saveQueue = saveQueue
			.catch(() => {})
			.then(async () => {
				await save(payload);
				lastSavedForm = nextSnapshot;
			});

		await saveQueue;
	},
	{ deep: true },
);

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
	padding: 14px 14px 12px 14px;
	display: flex;
	flex-direction: column;
	gap: 10px;
	overflow: hidden;
}

.panel-body {
	flex: 1;
	background: #ffffff;
	border: 1px solid #e5e7eb;
	border-radius: 14px;
	padding: 18px 20px;
	display: flex;
	flex-direction: column;
}

.panel-body--system {
	overflow: hidden;
}

.panel-body--reminder {
	overflow-y: auto;
}

.content-stack {
	display: flex;
	flex-direction: column;
	gap: 16px;
}

.setting-row {
	min-height: 44px;
	display: flex;
	align-items: center;
	gap: 12px;
	padding: 0;
}

.setting-copy {
	min-width: 0;
	flex: 1;
}

.setting-title {
	font-size: 14px;
	font-weight: 700;
	color: #374151;
}

.setting-desc {
	margin-top: 2px;
	font-size: 12px;
	line-height: 1.35;
	color: #9ca3af;
}

.setting-control {
	display: flex;
	align-items: center;
	justify-content: flex-end;
	flex-shrink: 0;
}

.setting-control--switch,
.setting-control--path {
	width: 300px;
	max-width: 300px;
}

.path-input {
	width: 100%;
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

.reminder-sections {
	display: flex;
	flex-direction: column;
	gap: 18px;
}

.section-block {
	display: flex;
	flex-direction: column;
}

.section-title {
	margin: 0 0 12px;
	font-size: 14px;
	font-weight: 700;
	color: #374151;
}

.section-divider {
	height: 1px;
	background: #eceff3;
}

.slider-list {
	display: flex;
	flex-direction: column;
	gap: 12px;
	margin-top: 12px;
}

.slider-row {
	min-height: 32px;
	display: flex;
	align-items: center;
	gap: 12px;
}

.slider-label {
	width: 72px;
	flex-shrink: 0;
	font-size: 13px;
	font-weight: 600;
	color: #374151;
}

.slider-control {
	flex: 1;
	min-width: 0;
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
	min-height: 44px;
	flex-shrink: 0;
	display: flex;
	align-items: center;
	gap: 12px;
}

.guide-inline {
	flex: 1;
	min-width: 0;
	font-size: 12px;
	color: #9ca3af;
	line-height: 1.5;
}

.footer-actions {
	flex-shrink: 0;
	display: flex;
	align-items: center;
}

.secondary-button {
	height: 36px;
	min-width: 92px;
	padding: 0 16px;
	border: 1px solid #d1d5db;
	border-radius: 10px;
	background: #ffffff;
	color: #4b5563;
	font-size: 13px;
	font-weight: 600;
	white-space: nowrap;
	cursor: pointer;
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
