<script setup>
import { computed, ref } from "vue";

import EmptyState from "./EmptyState.vue";

const props = defineProps({
  title: String,
  subtitle: String,
  sources: {
    type: Array,
    required: true,
  },
  inputValue: String,
  statusText: String,
  busy: Boolean,
  t: {
    type: Function,
    required: true,
  },
});

const emit = defineEmits(["update:inputValue", "add", "remove"]);
const filterValue = ref("");

const filteredSources = computed(() => {
  const query = filterValue.value.trim().toLowerCase();
  if (!query) {
    return props.sources;
  }

  return props.sources.filter((source) => source.toLowerCase().includes(query));
});
</script>

<template>
  <section class="sources panel">
    <div class="sources__header">
      <div>
        <h2>{{ title }}</h2>
        <p>{{ subtitle }}</p>
      </div>
    </div>

    <div class="sources__toolbar">
      <input
        :value="inputValue"
        class="input"
        :placeholder="t('settings.sourcePlaceholder')"
        @input="emit('update:inputValue', $event.target.value)"
      />
      <button class="button button--primary" type="button" :disabled="busy || !inputValue?.trim()" @click="emit('add')">
        {{ t("settings.addSource") }}
      </button>
    </div>

    <input
      v-model="filterValue"
      class="input"
      :placeholder="t('sources.search')"
    />

    <p v-if="statusText" class="sources__status">{{ statusText }}</p>

    <div class="sources__panel panel panel--soft">
      <div v-if="filteredSources.length" class="sources__list">
        <article v-for="source in filteredSources" :key="source" class="source-row">
          <code>{{ source }}</code>
          <button class="button button--ghost" type="button" @click="emit('remove', source)">
            {{ t("sources.remove") }}
          </button>
        </article>
      </div>
      <EmptyState
        v-else
        :title="sources.length ? t('common.noMatches') : t('sources.emptyTitle')"
        :copy="sources.length ? t('sources.search') : t('sources.empty')"
      />
    </div>
  </section>
</template>

<style scoped>
.sources {
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.8rem;
  overflow: hidden;
}

.sources__header h2,
.sources__header p {
  margin: 0;
}

.sources__header h2 {
  font-size: 0.96rem;
  font-weight: 600;
}

.sources__header p {
  color: var(--muted);
  font-size: 0.82rem;
}

.sources__toolbar {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 0.75rem;
  align-items: center;
}

.sources__status {
  margin: 0;
  color: var(--muted);
  font-size: 0.88rem;
}

.sources__panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding: 0.75rem 0.8rem;
  border-radius: 0.8rem;
}

.sources__list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1 1 auto;
  height: 100%;
  min-height: 0;
  overflow: auto;
  padding-right: 0.1rem;
}

.source-row {
  flex: 0 0 auto;
  display: flex;
  gap: 0.75rem;
  justify-content: space-between;
  align-items: center;
  padding: 0.6rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  background: var(--panel);
  min-width: 0;
  overflow: hidden;
}

.source-row code {
  overflow-wrap: anywhere;
  color: var(--text);
  font-size: 0.8rem;
  min-width: 0;
  white-space: normal;
  flex: 1;
}

@media (max-width: 720px) {
  .sources__toolbar {
    grid-template-columns: 1fr;
  }

  .source-row {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
