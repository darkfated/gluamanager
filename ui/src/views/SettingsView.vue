<script setup>
import SourceList from "../components/SourceList.vue";
import { useAppStore } from "../composables/useAppStore.js";

const store = useAppStore();
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
    </div>

    <SourceList
      :title="store.t('settings.sourceTitle')"
      :subtitle="store.state.sourcesStatusMessage || ''"
      :sources="store.state.sources"
      :input-value="store.state.sourceInput"
      :status-text="''"
      :busy="store.state.busy"
      :t="store.t"
      @update:input-value="store.setSourceInput"
      @add="store.addSource"
      @remove="store.removeSource"
    />
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
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 0.75fr);
  align-items: start;
}

.settings-panel {
  display: grid;
  gap: 0.75rem;
  align-content: stretch;
  grid-auto-rows: min-content;
  min-width: 0;
  overflow: hidden;
  padding: 0.8rem;
  height: 100%;
}

.settings-panel__header h2,
.settings-stat span,
.settings-stat strong {
  margin: 0;
}

.settings-panel__header h2 {
  font-size: 0.96rem;
  font-weight: 600;
}

.settings-stat span,
.settings-select span {
  color: var(--muted);
  overflow-wrap: anywhere;
  font-size: 0.82rem;
}

.settings-stats {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.settings-stat {
  min-height: 0;
  display: grid;
  gap: 0.35rem;
  align-content: start;
  padding: 0.75rem 0.8rem;
  overflow: hidden;
  border-radius: 0.8rem;
}

.settings-stat strong {
  font-size: 0.9rem;
  line-height: 1.3;
  overflow-wrap: anywhere;
  font-weight: 600;
}

.settings-panel__button {
  justify-self: start;
}

.settings-select {
  display: grid;
  padding: 0.75rem 0.8rem;
  border-radius: 0.8rem;
  align-self: start;
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

@media (max-width: 980px) {
  .settings-top,
  .settings-stats {
    grid-template-columns: 1fr;
  }
}
</style>
