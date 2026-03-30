<script setup>
import { computed } from "vue";

import ReadmeContent from "./ReadmeContent.vue";

const props = defineProps({
  open: Boolean,
  addon: Object,
  targetType: String,
  checking: Boolean,
  tab: String,
  readme: Object,
  readmeLoading: Boolean,
  installPlan: Object,
  installPlanLoading: Boolean,
  rootPath: String,
  busy: Boolean,
  t: {
    type: Function,
    required: true,
  },
  tf: {
    type: Function,
    required: true,
  },
  repositoryLink: {
    type: Function,
    required: true,
  },
});

const emit = defineEmits([
  "close",
  "tab",
  "open-external",
  "install-preview",
  "install-confirm",
  "install-cancel",
  "update-addon",
  "rollback-addon",
  "remove-addon",
]);

const isInstalled = computed(() => props.targetType === "installed");
const repoHref = computed(() => props.repositoryLink(props.addon?.repositoryUrl));
const versionsEqual = computed(() => {
  if (!props.addon?.remoteVersion || !props.addon?.version) {
    return false;
  }
  return props.addon.version.trim() === props.addon.remoteVersion.trim();
});
const canUpdate = computed(
  () =>
    isInstalled.value &&
    !props.checking &&
    props.addon?.hasUpdate &&
    props.addon?.remoteVersion &&
    !versionsEqual.value,
);
const canRollback = computed(
  () =>
    isInstalled.value &&
    !props.checking &&
    Boolean(props.addon?.remoteVersion) &&
    !versionsEqual.value &&
    !props.addon?.hasError,
);
const canInstall = computed(
  () => !isInstalled.value && !props.addon?.installed && !props.busy && !props.installPlan,
);
const canRemove = computed(() => isInstalled.value && !props.busy && Boolean(props.addon?.addonPath));
</script>

<template>
  <Teleport to="body">
    <transition name="fade">
      <div v-if="open && addon" class="modal-shell" @click.self="emit('close')">
        <div class="modal-card panel">
          <header class="modal-card__header">
            <div class="modal-card__headline">
              <h3>{{ addon.name || t("common.notSpecified") }}</h3>
            </div>
            <button class="icon-button" type="button" @click="emit('close')">x</button>
          </header>

          <div class="modal-card__tabs">
            <button
              class="tab-button"
              :class="{ 'is-active': tab === 'info' }"
              type="button"
              @click="emit('tab', 'info')"
            >
              {{ t("modal.infoTab") }}
            </button>
            <button
              class="tab-button"
              :class="{ 'is-active': tab === 'readme' }"
              type="button"
              @click="emit('tab', 'readme')"
            >
              {{ t("modal.readmeTab") }}
            </button>
          </div>

          <div v-if="tab === 'info'" class="modal-card__body modal-card__info-grid">
            <div class="info-stack">
              <article class="detail-row">
                <span>{{ t("modal.installedVersion") }}</span>
                <strong>{{ addon.version || t("common.notSpecified") }}</strong>
              </article>
              <article class="detail-row">
                <span>{{ t("modal.remoteVersion") }}</span>
                <strong>
                  {{
                    checking
                      ? t("modal.checking")
                      : addon.remoteVersion || t("modal.notChecked")
                  }}
                </strong>
              </article>
              <article class="detail-row">
                <span>{{ t("modal.author") }}</span>
                <strong>{{ addon.author || t("common.notSpecified") }}</strong>
              </article>
              <article class="detail-row">
                <span>{{ t("modal.state") }}</span>
                <strong>{{ checking ? t("modal.checking") : addon.status }}</strong>
              </article>
              <article class="detail-row">
                <span>{{ t("modal.folder") }}</span>
                <strong>{{ isInstalled ? addon.addonPath : rootPath || t("common.notSpecified") }}</strong>
              </article>
              <article class="detail-row">
                <span>{{ t("modal.branch") }}</span>
                <strong>{{ addon.branch || t("common.notSpecified") }}</strong>
              </article>
            </div>

            <div class="info-stack">
              <article class="detail-panel panel panel--soft">
                <span>{{ t("modal.description") }}</span>
                <p class="detail-panel__description">{{ addon.description || t("addons.noDescription") }}</p>
              </article>

              <article class="detail-panel panel panel--soft">
                <span>{{ t("modal.repository") }}</span>
                <button
                  v-if="repoHref"
                  class="link-button"
                  type="button"
                  @click="emit('open-external', repoHref)"
                >
                  {{ addon.repositoryUrl }}
                </button>
                <p v-else>{{ addon.repositoryUrl || t("common.notSpecified") }}</p>
              </article>

              <article class="detail-panel panel panel--soft">
                <span>{{ t("modal.preserve") }}</span>
                <div class="tag-list">
                  <span
                    v-for="item in addon.preserve?.length ? addon.preserve : [t('modal.noProtectedPaths')]"
                    :key="item"
                    class="chip"
                  >
                    {{ item }}
                  </span>
                </div>
              </article>

              <article class="detail-panel panel panel--soft">
                <span>{{ t("modal.dependencies") }}</span>
                <div class="tag-list">
                  <span
                    v-for="item in addon.dependencies?.length
                      ? addon.dependencies.map((dependency) => `${dependency.url}${dependency.branch ? `#${dependency.branch}` : ''}`)
                      : [t('modal.noDependencies')]"
                    :key="item"
                    class="chip"
                  >
                    {{ item }}
                  </span>
                </div>
              </article>
            </div>
          </div>

          <div v-else class="modal-card__body">
            <ReadmeContent
              :content="readme"
              :loading="readmeLoading"
              :empty-text="t('modal.readmeEmpty')"
              :loading-text="t('modal.readmeLoading')"
            />
          </div>

          <div class="modal-card__footer">
            <div class="modal-card__actions">
              <button
                v-if="canRemove"
                class="button button--danger"
                type="button"
                :disabled="busy"
                @click="emit('remove-addon')"
              >
                {{ t("addons.remove") }}
              </button>
              <button
                v-if="canRollback"
                class="button button--ghost"
                type="button"
                :disabled="busy"
                @click="emit('rollback-addon')"
              >
                {{ t("addons.rollback") }}
              </button>
              <button
                v-if="canInstall"
                class="button button--primary"
                type="button"
                :disabled="busy || installPlanLoading"
                @click="emit('install-preview')"
              >
                {{ t("addons.install") }}
              </button>
              <button
                v-if="canUpdate"
                class="button button--primary"
                type="button"
                :disabled="busy"
                @click="emit('update-addon')"
              >
                {{ tf("modal.updateRange", { from: addon.version, to: addon.remoteVersion || addon.version }) }}
              </button>
            </div>
          </div>

          <section v-if="installPlan" class="install-plan panel panel--soft">
            <div class="install-plan__copy">
              <h4>{{ t("install.confirmTitle") }}</h4>
              <p>{{ tf("install.confirmText", { root: installPlan.rootName }) }}</p>
            </div>
            <div class="install-plan__list">
              <div v-for="item in installPlan.items" :key="`${item.repositoryUrl}-${item.branch}`" class="install-plan__item">
                <strong>{{ item.name || item.repositoryUrl }}</strong>
                <span>{{ item.branch }}</span>
              </div>
            </div>
            <div class="install-plan__actions">
              <button class="button button--ghost" type="button" @click="emit('install-cancel')">
                {{ t("install.cancel") }}
              </button>
              <button class="button button--primary" type="button" :disabled="busy" @click="emit('install-confirm')">
                {{ t("install.accept") }}
              </button>
            </div>
          </section>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
.modal-shell {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 1rem;
  background: rgba(6, 10, 16, 0.76);
  z-index: 100;
}

.modal-card {
  width: min(72rem, 100%);
  max-height: calc(100vh - 2rem);
  overflow: hidden;
  display: grid;
  gap: 0.7rem;
  padding: 0.8rem;
  border-radius: 0.95rem;
  grid-template-rows: auto auto minmax(0, 1fr) auto auto;
}

.modal-card__header,
.modal-card__tabs,
.modal-card__footer,
.install-plan__actions {
  display: flex;
  gap: 0.65rem;
  justify-content: space-between;
  align-items: center;
}

.modal-card__headline {
  min-width: 0;
}

.modal-card__eyebrow {
  margin: 0 0 0.3rem;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.78rem;
}

.modal-card__header h3,
.modal-card__header span,
.detail-panel p,
.install-plan__copy p {
  margin: 0;
  overflow-wrap: anywhere;
}

.modal-card__header h3 {
  font-size: 0.9rem;
  font-weight: 600;
  line-height: 1.2;
}

.modal-card__header span {
  color: var(--muted);
  font-size: 0.76rem;
  line-height: 1.25;
}

.modal-card__tabs {
  justify-content: flex-start;
}

.tab-button {
  padding: 0.44rem 0.78rem;
  border-radius: 0.65rem;
  border: 1px solid rgba(148, 163, 184, 0.16);
  background: transparent;
  color: var(--muted);
  font-size: 0.78rem;
  line-height: 1;
}

.tab-button.is-active {
  color: var(--text);
  border-color: rgba(125, 211, 252, 0.34);
  background: rgba(14, 165, 233, 0.1);
}

.modal-card__info-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
  gap: 0.75rem;
  min-height: 0;
}

.info-stack {
  display: grid;
  gap: 0.75rem;
  min-height: 0;
  min-width: 0;
}

.detail-row,
.detail-panel {
  display: grid;
  gap: 0.28rem;
  min-width: 0;
}

.detail-panel {
  border-radius: 0.75rem;
  box-shadow: none;
  overflow: hidden;
  padding: 0.75rem 0.8rem;
}

.detail-row span,
.detail-panel span {
  color: var(--muted);
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.detail-row strong,
.detail-panel p,
.install-plan__item strong,
.install-plan__item span {
  font-size: 0.82rem;
  line-height: 1.3;
}

.link-button {
  padding: 0;
  border: 0;
  background: none;
  color: var(--accent-2);
  text-align: left;
  cursor: pointer;
  overflow-wrap: anywhere;
  font-size: 0.8rem;
  line-height: 1.3;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  min-width: 0;
}

.detail-panel__description {
  max-height: 7.5rem;
  overflow: auto;
  padding-right: 0.2rem;
  font-size: 0.82rem;
  line-height: 1.35;
}

.modal-card__body {
  min-height: 0;
  overflow: hidden;
}

.modal-card__actions {
  display: flex;
  gap: 0.6rem;
  flex-wrap: wrap;
  justify-content: flex-end;
  width: 100%;
}

.install-plan {
  display: grid;
  gap: 0.75rem;
  padding: 0.75rem 0.8rem;
  border-radius: 0.8rem;
}

.install-plan__copy h4 {
  margin: 0 0 0.3rem;
  font-size: 0.9rem;
  font-weight: 600;
}

.install-plan__copy p {
  color: var(--muted);
  font-size: 0.82rem;
  line-height: 1.35;
}

.install-plan__list {
  display: grid;
  gap: 0.5rem;
  max-height: 12.5rem;
  overflow: auto;
  padding-right: 0.12rem;
}

.install-plan__item {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  align-items: center;
  padding: 0.6rem 0.75rem;
  border-radius: 0.75rem;
  background: #161b23;
  border: 1px solid rgba(148, 163, 184, 0.12);
  font-size: 0.82rem;
}

.icon-button {
  width: 2.1rem;
  height: 2.1rem;
  border-radius: 999px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: rgba(255, 255, 255, 0.04);
  color: var(--text);
  font-size: 0.8rem;
}

@media (max-width: 860px) {
  .modal-card__info-grid {
    grid-template-columns: 1fr;
  }
}
</style>
