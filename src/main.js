import { createApp } from "vue";
import "./style.css";

/**
 * 根据当前窗口 hash 按需加载对应视图。
 * 返回值：Promise<VueComponent>
 */
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

	// 空壳窗口：此项目通过系统托盘运行，无独立 main 窗口
	const { default: App } = await import("./App.vue");
	return App;
}

/** 启动前端入口，避免顶层 await 影响生产构建 */
async function bootstrap() {
	const rootComponent = await loadRootComponent();
	createApp(rootComponent).mount("#app");
}

void bootstrap();
