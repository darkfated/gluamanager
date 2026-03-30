<script setup>
import { computed } from "vue";

import { initials } from "../utils/format.js";

const props = defineProps({
  addon: {
    type: Object,
    required: true,
  },
  mode: {
    type: String,
    required: true,
  },
  t: {
    type: Function,
    required: true,
  },
  tf: {
    type: Function,
    required: true,
  },
});

const statusKind = computed(() => {
  if (props.addon.hasError) {
    return "error";
  }
  if (props.addon.hasUpdate) {
    return "update";
  }
  if (props.addon.installed || props.mode === "installed") {
    return "ok";
  }
  return "neutral";
});

const subtitle = computed(() => {
  const author = props.addon.author || props.t("common.notSpecified");
  return props.tf("addons.authorShort", { author });
});

const statusText = computed(() => {
  if (props.addon.hasError) {
    return props.t("addons.badgeError");
  }
  if (props.addon.hasUpdate) {
    return props.t("addons.badgeUpdate");
  }
  if (props.addon.installed || props.mode === "installed") {
    return props.t("addons.installedState");
  }
  return "";
});

</script>

<template>
  <article class="addon-card panel panel--soft">
    <div class="addon-card__identity">
      <div class="addon-card__avatar">{{ initials(addon.name || addon.sourceUrl || addon.url || "GM") }}</div>
      <div class="addon-card__copy">
        <h3>{{ addon.name || t("common.notSpecified") }}</h3>
        <p>{{ subtitle }}</p>
      </div>
      <span class="addon-card__status" :data-kind="statusKind">{{ statusText }}</span>
    </div>
  </article>
</template>

<style scoped>
.addon-card {
  display: flex;
  align-items: center;
  min-height: 3.8rem;
  max-height: 3.8rem;
  padding: 0.48rem 0.72rem;
  border-radius: 0.9rem;
  box-shadow: none;
  min-width: 0;
  overflow: hidden;
}

.addon-card__identity {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 0.55rem;
  align-items: center;
  min-width: 0;
  width: 100%;
}

.addon-card__copy {
  min-width: 0;
  display: grid;
  gap: 0.08rem;
  align-content: center;
}

.addon-card__avatar {
  width: 2rem;
  height: 2rem;
  display: grid;
  place-items: center;
  align-self: center;
  border-radius: 0.7rem;
  background: rgba(14, 165, 233, 0.12);
  color: var(--text);
  font-size: 0.78rem;
  font-weight: 700;
}

.addon-card h3,
.addon-card p {
  margin: 0;
  min-width: 0;
  overflow-wrap: anywhere;
}

.addon-card h3 {
  font-size: 0.88rem;
  line-height: 1.2;
}

.addon-card__identity p {
  color: var(--muted);
  font-size: 0.78rem;
  line-height: 1.25;
}

.addon-card__status {
  color: var(--muted);
  font-size: 0.75rem;
  line-height: 1.2;
  text-align: right;
  justify-self: end;
  white-space: nowrap;
}

.addon-card__status[data-kind="ok"] {
  color: #a9b7c9;
}

.addon-card__status[data-kind="update"] {
  color: #8ba5ff;
}

.addon-card__status[data-kind="error"] {
  color: #fca5a5;
}

@media (max-width: 640px) {
  .addon-card__identity {
    grid-template-columns: auto 1fr;
  }

  .addon-card__status {
    grid-column: 1 / -1;
    justify-self: start;
    white-space: normal;
  }
}
</style>
