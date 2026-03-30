<script setup>
import { onMounted, onUnmounted } from "vue";
import { RouterLink, RouterView } from "vue-router";

import AddonModal from "./components/AddonModal.vue";
import { useAppStore } from "./composables/useAppStore.js";

const store = useAppStore();

function handleEscape(event) {
  if (event.key === "Escape" && store.state.modalOpen) {
    store.closeModal();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleEscape);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleEscape);
});
</script>

<template>
  <div class="app-frame">
    <div class="desktop-shell panel">
      <header class="desktop-header">
        <div class="desktop-titlebar__brand">
          <strong>{{ store.t("common.appName") }}</strong>
        </div>
        <nav class="desktop-tabs">
          <RouterLink to="/addons" class="desktop-tab">
            {{ store.t("nav.addons") }}
          </RouterLink>
          <RouterLink to="/settings" class="desktop-tab">
            {{ store.t("nav.settings") }}
          </RouterLink>
        </nav>
        <div class="desktop-titlebar__meta">
          {{ store.state.appVersion || "v0.2.0" }}
        </div>
      </header>

      <main class="app-main">
        <RouterView />
      </main>
    </div>

    <AddonModal
      :open="store.state.modalOpen"
      :addon="store.state.modalAddon"
      :target-type="store.state.modalTargetType"
      :checking="store.state.modalChecking"
      :tab="store.state.modalTab"
      :readme="store.state.modalReadme"
      :readme-loading="store.state.modalReadmeLoading"
      :install-plan="store.state.installPlan"
      :install-plan-loading="store.state.installPlanLoading"
      :root-path="store.state.rootPath"
      :busy="store.state.busy"
      :t="store.t"
      :tf="store.tf"
      :repository-link="store.repositoryLink"
      @close="store.closeModal"
      @tab="store.setModalTab"
      @open-external="store.openExternalUrl"
      @install-preview="store.requestInstallFromModal"
      @install-confirm="store.confirmInstallPlan"
      @install-cancel="store.closeInstallPlan"
      @update-addon="store.updateSelectedFromModal"
      @rollback-addon="store.rollbackSelectedFromModal"
      @remove-addon="store.removeSelectedFromModal"
    />
  </div>
</template>

<style scoped>
.desktop-shell {
  height: 100vh;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  padding: 0;
  overflow: hidden;
  border-radius: 0;
  border-left: 0;
  border-right: 0;
  border-top: 0;
}

.desktop-header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 0.85rem;
  align-items: center;
  min-height: 3.6rem;
  padding: 0.7rem 0.85rem;
  border-bottom: 1px solid rgba(148, 163, 184, 0.12);
  background: #181d25;
}

.desktop-titlebar__brand {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.desktop-titlebar__brand strong {
  line-height: 1.1;
  font-size: 0.95rem;
  font-weight: 600;
}

.desktop-titlebar__meta {
  color: var(--muted);
  font-size: 0.86rem;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.desktop-tabs {
  display: flex;
  justify-content: center;
  gap: 0.55rem;
}

.desktop-tab {
  padding: 0.48rem 0.85rem;
  border-radius: 0.65rem;
  color: var(--muted);
  text-decoration: none;
  border: 1px solid transparent;
  font-size: 0.84rem;
}

.desktop-tab.router-link-active {
  color: var(--text);
  background: #202631;
  border-color: rgba(148, 163, 184, 0.18);
}

.app-main {
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding: 0.75rem;
}

@media (max-width: 880px) {
  .desktop-header {
    grid-template-columns: 1fr;
    justify-items: start;
  }

  .desktop-tabs {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
}
</style>
