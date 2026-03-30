import { createRouter, createWebHashHistory } from "vue-router";

import AddonsView from "./views/AddonsView.vue";
import SettingsView from "./views/SettingsView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      redirect: "/addons",
    },
    {
      path: "/addons",
      name: "addons",
      component: AddonsView,
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView,
    },
  ],
});
