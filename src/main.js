import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";

if (process.env.NODE_ENV === "production") {
    document.addEventListener("contextmenu", (event) => event.preventDefault());
}
createApp(App).mount("#app");
