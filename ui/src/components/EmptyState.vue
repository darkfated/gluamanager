<script setup>
defineProps({
  title: String,
  copy: String,
});
</script>

<template>
  <div class="empty panel panel--soft">
    <strong>{{ title }}</strong>
    <p>{{ copy }}</p>
  </div>
</template>

<style scoped>
.empty {
  display: grid;
  gap: 0.6rem;
  min-height: 12rem;
  place-content: center;
  text-align: center;
}

.empty strong {
  font-size: 1.2rem;
}

.empty p {
  margin: 0;
  max-width: 30rem;
  color: var(--muted);
  overflow-wrap: anywhere;
}
</style>
