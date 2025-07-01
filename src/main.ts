import { createApp } from "vue";
// @ts-expect-error
import "@picocss/pico";
import "./style.css";
import App from "./App.vue";
import { useNotification } from "./composables/useNotification";

const app = createApp(App);

// 初始化通知权限
const { initializeNotifications } = useNotification();
initializeNotifications().catch(console.warn);

app.mount("#app");
