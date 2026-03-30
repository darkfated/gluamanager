<script setup>
import { ref } from "vue";

import AddonMetadataModal from "../components/AddonMetadataModal.vue";
import SourceList from "../components/SourceList.vue";
import { useAppStore } from "../composables/useAppStore.js";

const store = useAppStore();
const metadataOpen = ref(false);

function openMetadataBuilder() {
  metadataOpen.value = true;
}

function closeMetadataBuilder() {
  metadataOpen.value = false;
}
</script>

<template>
  <div class="desktop-page">
    <div class="settings-top">
      <section class="settings-panel panel">
        <header class="settings-panel__header">
          <h2>{{ store.t("settings.application") }}</h2>
        </header>

        <div class="settings-stats">
          <article class="settings-stat panel panel--soft">
            <span>{{ store.t("settings.currentVersion") }}</span>
            <strong>{{ store.state.appVersion || store.t("common.notSpecified") }}</strong>
          </article>
          <article class="settings-stat panel panel--soft">
            <span>{{ store.t("settings.availableUpdate") }}</span>
            <strong>
              {{
                store.state.appUpdate
                  ? store.tf("settings.updateVersion", { version: store.state.appUpdate.version })
                  : store.state.updaterEnabled
                    ? store.t("settings.noAppUpdate")
                    : store.t("settings.updaterDisabled")
              }}
            </strong>
          </article>
        </div>

        <button
          v-if="store.state.appUpdate"
          class="button button--primary settings-panel__button"
          type="button"
          :disabled="store.state.appUpdating"
          @click="store.installAppUpdate()"
        >
          {{ store.t("app.installUpdate") }}
        </button>
      </section>

      <section class="settings-panel panel">
        <header class="settings-panel__header">
          <h2>{{ store.t("settings.appearanceTitle") }}</h2>
        </header>

        <label class="settings-select panel panel--soft">
          <select
            class="select"
            :value="store.state.language"
            @change="store.setLanguage($event.target.value)"
          >
            <option value="en">English</option>
            <option value="ru">Русский</option>
          </select>
        </label>
      </section>

      <section class="settings-panel panel">
        <header class="settings-panel__header">
          <h2>{{ store.t("settings.metadataTitle") }}</h2>
        </header>

        <button
          class="button button--primary settings-panel__button settings-panel__button--full settings-panel__button--field"
          type="button"
          @click="openMetadataBuilder"
        >
          {{ store.t("settings.metadataButton") }}
        </button>
      </section>
    </div>

    <SourceList
      :title="store.t('settings.sourceTitle')"
      :subtitle="store.state.sourcesStatusMessage || store.t('sources.subtitle')"
      :sources="store.state.sources"
      :input-value="store.state.sourceInput"
      :status-text="''"
      :busy="store.state.busy"
      :t="store.t"
      @update:input-value="store.setSourceInput"
      @add="store.addSource"
      @remove="store.removeSource"
    />

    <AddonMetadataModal :open="metadataOpen" :t="store.t" @close="closeMetadataBuilder" />
  </div>
</template>

<style scoped>
.desktop-page {
  height: 100%;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 0.75rem;
  min-height: 0;
  overflow: hidden;
}

.settings-top,
.settings-stats {
  display: grid;
  gap: 0.75rem;
}

.settings-top {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  align-items: stretch;
}

.settings-panel {
  display: grid;
  gap: 0.5rem;
  grid-template-rows: auto minmax(0, 1fr) auto;
  align-content: start;
  min-width: 0;
  overflow: hidden;
  padding: 0.75rem;
  min-height: 0;
  height: 100%;
}

.settings-panel__header h2,
.settings-stat span,
.settings-stat strong {
  margin: 0;
}

.settings-panel__header h2 {
  font-size: 0.9rem;
  font-weight: 600;
  line-height: 1.25;
}

.settings-stat span,
.settings-select span {
  color: var(--muted);
  overflow-wrap: anywhere;
  font-size: 0.8rem;
}

.settings-stats {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
}

.settings-stat {
  min-height: 0;
  display: grid;
  gap: 0.3rem;
  align-content: start;
  padding: 0.55rem 0.65rem;
  overflow: hidden;
  border-radius: 0.8rem;
}

.settings-stat strong {
  font-size: 0.85rem;
  line-height: 1.3;
  overflow-wrap: anywhere;
  font-weight: 600;
}

.settings-panel__button {
  justify-self: start;
  min-height: 2rem;
  padding: 0.38rem 0.72rem;
  margin-top: auto;
}

.settings-panel__button--full {
  justify-self: stretch;
}

.settings-panel__button--field {
  width: 100%;
  min-height: 3.15rem;
  padding: 0.55rem 0.65rem;
  margin: 0.1rem 0;
}

.settings-select {
  display: grid;
  padding: 0.55rem 0.65rem;
  border-radius: 0.8rem;
  align-self: start;
  min-height: 0;
  height: 100%;
}

.settings-select .select {
  min-height: 2rem;
}

.settings-select .select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  padding-right: 2.4rem;
  background-image:
    linear-gradient(45deg, transparent 50%, var(--muted) 50%),
    linear-gradient(135deg, var(--muted) 50%, transparent 50%);
  background-position:
    calc(100% - 18px) calc(50% - 3px),
    calc(100% - 12px) calc(50% - 3px);
  background-size: 6px 6px, 6px 6px;
  background-repeat: no-repeat;
}

@media (max-width: 1180px) {
  .settings-top {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 980px) {
  .settings-top,
  .settings-stats {
    grid-template-columns: 1fr;
  }
}
</style>
