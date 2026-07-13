import { createApp } from "vue";
import "./element-plus.js";
import "./style.css";

async function loadRootComponent() {
	const hash = window.location.hash;

	if (hash.startsWith("#/popup")) {
		const { default: PopupWindow } = await import("./views/PopupWindow.vue");
		return PopupWindow;
	}

	if (hash.startsWith("#/alert")) {
		const { default: AlertWindow } = await import("./views/AlertWindow.vue");
		return AlertWindow;
	}

	if (hash.startsWith("#/settings")) {
		const { default: SettingsWindow } = await import(
			"./views/SettingsWindow.vue"
		);
		return SettingsWindow;
	}
	return { template: "<div />" };
}

/** 启动前端入口，避免顶层 await 影响生产构建 */
async function bootstrap() {
	const rootComponent = await loadRootComponent();
	createApp(rootComponent).mount("#app");
}

void bootstrap();
