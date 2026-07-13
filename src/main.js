import { createApp } from "vue";
import "./style.css";

// 根据 URL hash 决定渲染哪个窗口视图，避免引入完整路由库
const hash = window.location.hash;

let rootComponent;

if (hash.startsWith("#/popup")) {
	const { default: PopupWindow } = await import("./views/PopupWindow.vue");
	rootComponent = PopupWindow;
} else if (hash.startsWith("#/alert")) {
	const { default: AlertWindow } = await import("./views/AlertWindow.vue");
	rootComponent = AlertWindow;
} else {
	// 兜底：空壳（此项目无独立 main 窗口）
	const { default: App } = await import("./App.vue");
	rootComponent = App;
}

createApp(rootComponent).mount("#app");
