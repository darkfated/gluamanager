<script setup>
import { computed, ref } from "vue";
import { useRouter } from "vue-router";

import AddonCard from "../components/AddonCard.vue";
import EmptyState from "../components/EmptyState.vue";
import { useAppStore } from "../composables/useAppStore.js";

const store = useAppStore();
const router = useRouter();
const updateCount = store.updateCount;
const installedQuery = ref("");
const availableQuery = ref("");

function matchesAddon(addon, query) {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return true;
  }

  return [
    addon.name,
    addon.author,
    addon.description,
    addon.repositoryUrl,
    addon.branch,
  ]
    .filter(Boolean)
    .some((value) => String(value).toLowerCase().includes(normalized));
}

const filteredInstalledAddons = computed(() =>
  store.state.installedAddons.filter((addon) => matchesAddon(addon, installedQuery.value)),
);

const filteredAvailableAddons = computed(() =>
  store.state.availableAddons.filter((addon) => matchesAddon(addon, availableQuery.value)),
);

function openSettingsForSources() {
  router.push("/settings");
}
</script>

<template>
  <div class="desktop-page">
    <section class="workspace-toolbar panel">
      <div class="workspace-toolbar__main">
        <label class="workspace-field">
          <input
            :value="store.state.rootPath"
            class="input"
            :placeholder="store.t('workspace.placeholder')"
            @input="store.setRootPath($event.target.value)"
            @blur="store.saveTypedRootPath()"
          />
        </label>

        <div class="workspace-toolbar__actions">
          <button
            class="button button--ghost button--toolbar"
            type="button"
            :disabled="store.state.busy"
            @click="store.chooseFolderAction()"
          >
            {{ store.t("workspace.folder") }}
          </button>
          <button
            class="button button--primary button--toolbar"
            type="button"
            :disabled="store.state.busy"
            @click="store.refreshAll()"
          >
            {{ store.t("workspace.scan") }}
          </button>
        </div>
      </div>

    </section>

    <section
      v-if="store.state.appUpdate && !store.state.appUpdateDismissed"
      class="desktop-banner panel"
    >
      <div class="desktop-banner__copy">
        <strong>{{ store.tf("app.updateTitle", { version: store.state.appUpdate.version }) }}</strong>
        <span>{{ store.state.appUpdate.notes || store.t("app.updateReady") }}</span>
      </div>
      <div class="desktop-banner__actions">
        <button class="button button--ghost" type="button" @click="store.dismissAppUpdate()">
          {{ store.t("app.dismissUpdate") }}
        </button>
        <button
          class="button button--primary"
          type="button"
          :disabled="store.state.appUpdating"
          @click="store.installAppUpdate()"
        >
          {{ store.t("app.installUpdate") }}
        </button>
      </div>
    </section>

    <div class="desktop-columns">
      <section class="desktop-pane panel">
        <header class="desktop-pane__header">
          <h2>{{ store.t("addons.installedTitle") }}</h2>
          <div class="desktop-pane__actions">
            <button
              class="button button--ghost"
              type="button"
              :disabled="store.state.busy"
              @click="store.refreshInstalled()"
            >
              {{ store.t("addons.check") }}
            </button>
            <button
              class="button button--primary"
              type="button"
              :disabled="store.state.busy || updateCount === 0"
              @click="store.updateAll()"
            >
              {{
                updateCount
                  ? store.tf("addons.updateAllCount", { count: updateCount })
                  : store.t("addons.updateAll")
              }}
            </button>
          </div>
        </header>

        <input
          v-model="installedQuery"
          class="input"
          :placeholder="store.t('addons.searchInstalled')"
        />

        <div v-if="filteredInstalledAddons.length" class="desktop-pane__body addon-list">
          <div
            v-for="addon in filteredInstalledAddons"
            :key="addon.addonPath"
            class="addon-list__item"
            @click="store.openAddonModal(addon.addonPath)"
          >
            <AddonCard
              :addon="addon"
              mode="installed"
              :t="store.t"
              :tf="store.tf"
            />
          </div>
        </div>
        <EmptyState
          v-else
          :title="store.state.installedAddons.length ? store.t('common.noMatches') : store.t('addons.emptyTitle')"
          :copy="store.state.installedAddons.length ? store.t('addons.searchInstalled') : store.state.rootPath ? store.t('addons.emptyWithFolder') : store.t('addons.empty')"
        />
      </section>

      <section class="desktop-pane panel">
        <header class="desktop-pane__header">
          <h2>{{ store.t("addons.availableTitle") }}</h2>
          <div class="desktop-pane__actions">
            <button class="button button--ghost" type="button" @click="openSettingsForSources">
              {{ store.t("settings.sourceTitle") }}
            </button>
          </div>
        </header>

        <input
          v-model="availableQuery"
          class="input"
          :placeholder="store.t('addons.searchAvailable')"
        />

        <div v-if="filteredAvailableAddons.length" class="desktop-pane__body addon-list">
          <div
            v-for="addon in filteredAvailableAddons"
            :key="`${addon.repositoryUrl}-${addon.branch}`"
            class="addon-list__item"
            @click="store.openAvailableModal(addon.repositoryUrl, addon.branch)"
          >
            <AddonCard
              :addon="addon"
              mode="available"
              :t="store.t"
              :tf="store.tf"
            />
          </div>
        </div>
        <EmptyState
          v-else
          :title="store.state.availableAddons.length ? store.t('common.noMatches') : store.state.sources.length ? store.t('addons.availableEmptyTitle') : store.t('addons.availableNoSourcesTitle')"
          :copy="store.state.availableAddons.length ? store.t('addons.searchAvailable') : store.state.sources.length ? store.t('addons.availableEmpty') : store.t('addons.availableNoSources')"
        />
      </section>
    </div>
  </div>
</template>

<style scoped>
.desktop-page {
  height: 100%;
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 0.75rem;
  min-height: 0;
}

.workspace-toolbar,
.workspace-toolbar__main,
.workspace-toolbar__actions,
.workspace-toolbar__stats,
.desktop-banner,
.desktop-banner__actions,
.desktop-pane__header,
.desktop-pane__actions {
  display: flex;
  gap: 0.75rem;
}

.workspace-toolbar,
.desktop-banner,
.desktop-pane__header {
  align-items: center;
  justify-content: space-between;
}

.workspace-toolbar {
  flex-wrap: wrap;
  padding: 0.75rem 0.85rem;
}

.workspace-toolbar__main {
  flex: 1;
  min-width: min(42rem, 100%);
}

.workspace-field {
  flex: 1;
  min-width: 16rem;
}

.desktop-banner {
  padding: 0.72rem 0.85rem;
  background: var(--panel);
}

.desktop-banner__copy {
  display: grid;
  gap: 0.15rem;
}

.desktop-banner__copy strong {
  font-size: 0.88rem;
  font-weight: 600;
  line-height: 1.2;
}

.desktop-banner__copy span {
  color: var(--muted);
  font-size: 0.8rem;
  line-height: 1.3;
}

.desktop-columns {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 0.75rem;
  overflow: hidden;
  align-items: start;
}

.desktop-pane {
  min-height: 0;
  min-width: 0;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  overflow: hidden;
}

.desktop-pane__header {
  align-items: center;
  gap: 0.75rem;
}

.desktop-pane__header h2 {
  margin: 0;
  font-size: 0.96rem;
  font-weight: 600;
}

.desktop-pane__body {
  min-height: 0;
  min-width: 0;
  flex: 1 1 auto;
  overflow: hidden;
}

.addon-list {
  display: grid;
  gap: 0.5rem;
  overflow: auto;
  padding-right: 0.15rem;
  align-content: start;
}

.addon-list__item {
  cursor: pointer;
  min-width: 0;
}

.button--toolbar {
  min-width: 8.5rem;
}

@media (max-width: 1080px) {
  .desktop-page {
    grid-template-rows: auto auto auto;
  }

  .desktop-columns {
    grid-template-columns: 1fr;
    overflow: auto;
  }
}

@media (max-width: 760px) {
  .workspace-toolbar__main,
  .workspace-toolbar__actions {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
