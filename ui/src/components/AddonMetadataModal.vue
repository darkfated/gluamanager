<script setup>
import { computed, reactive, ref, watch } from "vue";

const props = defineProps({
  open: Boolean,
  t: {
    type: Function,
    required: true,
  },
});

const emit = defineEmits(["close"]);

const defaults = {
  name: "",
  description: "",
  author: "",
  version: "1.0.0",
  url: "",
  preserve: "",
  dependencies: "",
};

const form = reactive({ ...defaults });
const copied = ref(false);

const preserveList = computed(() => splitLines(form.preserve));
const dependencyList = computed(() => splitLines(form.dependencies));
const manifest = computed(() => ({
  info: {
    name: form.name.trim(),
    description: form.description.trim(),
    author: form.author.trim(),
  },
  version: form.version.trim(),
  url: form.url.trim(),
  preserve: preserveList.value,
  dependencies: dependencyList.value,
}));
const manifestText = computed(() => JSON.stringify(manifest.value, null, 2));
const canCopy = computed(() => Boolean(form.name.trim() && form.version.trim() && form.url.trim()));

watch(
  () => props.open,
  (open) => {
    if (open) {
      copied.value = false;
    }
  },
  { immediate: true },
);

function splitLines(value) {
  return value
    .split(/\r?\n/)
    .map((item) => item.trim())
    .filter(Boolean);
}

async function copyManifest() {
  if (!canCopy.value) {
    return;
  }

  try {
    await navigator.clipboard.writeText(manifestText.value);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1400);
  } catch {
    copied.value = false;
  }
}

function clearForm() {
  Object.assign(form, defaults);
  copied.value = false;
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="metadata-shell" @click.self="emit('close')">
      <div class="metadata-card panel">
        <header class="metadata-card__header">
          <div class="metadata-card__headline">
            <h3>{{ t("metadata.title") }}</h3>
          </div>
          <button class="icon-button" type="button" @click="emit('close')">x</button>
        </header>

        <div class="metadata-card__body">
          <section class="metadata-form panel panel--soft">
            <div class="metadata-field">
              <span>{{ t("metadata.name") }}</span>
              <input v-model="form.name" class="input" type="text" :placeholder="t('metadata.namePlaceholder')" />
            </div>

            <div class="metadata-field">
              <span>{{ t("metadata.version") }}</span>
              <input v-model="form.version" class="input" type="text" :placeholder="t('metadata.versionPlaceholder')" />
            </div>

            <div class="metadata-field">
              <span>{{ t("metadata.author") }}</span>
              <input v-model="form.author" class="input" type="text" :placeholder="t('metadata.authorPlaceholder')" />
            </div>

            <div class="metadata-field metadata-field--wide">
              <span>{{ t("metadata.url") }}</span>
              <input v-model="form.url" class="input" type="url" :placeholder="t('metadata.urlPlaceholder')" />
            </div>

            <div class="metadata-field metadata-field--wide">
              <span>{{ t("metadata.description") }}</span>
              <textarea
                v-model="form.description"
                class="input metadata-textarea"
                :placeholder="t('metadata.descriptionPlaceholder')"
              ></textarea>
            </div>

            <div class="metadata-field">
              <span>{{ t("metadata.preserve") }}</span>
              <textarea
                v-model="form.preserve"
                class="input metadata-textarea"
                :placeholder="t('metadata.linesPlaceholder')"
              ></textarea>
            </div>

            <div class="metadata-field">
              <span>{{ t("metadata.dependencies") }}</span>
              <textarea
                v-model="form.dependencies"
                class="input metadata-textarea"
                :placeholder="t('metadata.linesPlaceholder')"
              ></textarea>
            </div>
          </section>

          <section class="metadata-preview panel panel--soft">
            <div class="metadata-preview__header">
              <h4>{{ t("metadata.previewTitle") }}</h4>
            </div>
            <pre class="metadata-preview__code">{{ manifestText }}</pre>
          </section>
        </div>

        <footer class="metadata-card__footer">
          <button class="button button--ghost" type="button" @click="clearForm()">
            {{ t("metadata.clear") }}
          </button>
          <button class="button button--primary" type="button" :disabled="!canCopy" @click="copyManifest">
            {{ copied ? t("metadata.copied") : t("metadata.copy") }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.metadata-shell {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 1rem;
  background: rgba(6, 10, 16, 0.82);
  z-index: 120;
  user-select: none;
}

.metadata-card,
.metadata-card * {
  user-select: none;
}

.metadata-card {
  width: min(78rem, 100%);
  max-height: calc(100vh - 2rem);
  height: min(42rem, calc(100vh - 2rem));
  overflow: hidden;
  display: grid;
  gap: 0.8rem;
  padding: 0.85rem;
  border-radius: 0.95rem;
  grid-template-rows: auto minmax(0, 1fr) auto;
}

.metadata-card__header,
.metadata-card__footer,
.metadata-preview__header {
  display: flex;
  gap: 0.75rem;
  justify-content: space-between;
  align-items: center;
}

.metadata-card__headline {
  min-width: 0;
}

.metadata-card__headline h3,
.metadata-card__headline p,
.metadata-preview__header h4,
.metadata-preview__header p {
  margin: 0;
}

.metadata-card__headline h3 {
  font-size: 0.96rem;
  font-weight: 600;
}

.metadata-card__body {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.8fr);
  gap: 0.8rem;
  min-height: 0;
  align-items: stretch;
}

.metadata-form,
.metadata-preview {
  min-height: 0;
  overflow: hidden;
}

.metadata-form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.65rem;
  padding: 0.75rem;
  border-radius: 0.85rem;
  align-content: start;
  height: 100%;
  overflow: auto;
}

.metadata-field {
  display: grid;
  gap: 0.3rem;
  min-width: 0;
}

.metadata-field--wide {
  grid-column: 1 / -1;
}

.metadata-field span {
  color: var(--muted);
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.metadata-textarea {
  min-height: 5.5rem;
  resize: none;
}

.metadata-preview {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 0.45rem;
  padding: 0.75rem;
  border-radius: 0.85rem;
  align-self: stretch;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.metadata-preview__header {
  align-items: center;
}

.metadata-preview__header h4 {
  font-size: 0.88rem;
  font-weight: 600;
}

.metadata-preview__code {
  margin: 0;
  padding: 0.8rem;
  min-height: 0;
  height: 100%;
  overflow: auto;
  border-radius: 0.8rem;
  background: #131821;
  color: var(--text);
  font-family:
    "JetBrains Mono",
    "SFMono-Regular",
    monospace;
  font-size: 0.78rem;
  line-height: 1.45;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  user-select: text;
}

.metadata-card__footer {
  justify-content: flex-end;
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

@media (max-width: 1080px) {
  .metadata-card__body {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .metadata-form {
    grid-template-columns: 1fr;
  }
}
</style>
