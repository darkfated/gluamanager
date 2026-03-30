<script setup>
import { computed } from "vue";

import { renderMarkdown } from "../utils/markdown.js";

const props = defineProps({
  content: Object,
  loading: Boolean,
  emptyText: String,
  loadingText: String,
});

const html = computed(() => {
  if (props.loading) {
    return `<div class="readme-state">${props.loadingText}</div>`;
  }

  if (!props.content) {
    return `<div class="readme-state">${props.emptyText}</div>`;
  }

  return renderMarkdown(props.content.content, {
    baseUrl: props.content.baseUrl,
    localBasePath: props.content.localBasePath,
  });
});
</script>

<template>
  <div class="readme-content" v-html="html"></div>
</template>

<style scoped>
.readme-content {
  height: 100%;
  min-height: 0;
  overflow: auto;
  padding-right: 0.2rem;
  overflow-x: hidden;
  font-size: 0.8rem;
  line-height: 1.32;
}
</style>
