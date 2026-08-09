import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { useProjectsStore } from "./stores/projects";
import "./style.css";

const pinia = createPinia();
const app = createApp(App);
app.use(pinia);

useProjectsStore(pinia)
  .load()
  .finally(() => {
    app.mount("#app");
  });
