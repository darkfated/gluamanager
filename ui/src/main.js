import { createApp } from "vue";

import App from "./App.vue";
import { router } from "./router.js";
import { createAppStore, appStoreKey } from "./composables/useAppStore.js";
import "./assets/main.css";

const app = createApp(App);
const store = createAppStore();

app.provide(appStoreKey, store);
app.use(router);

store.initialize();

app.mount("#app");
