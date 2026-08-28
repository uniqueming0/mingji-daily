<script setup lang="ts">
defineProps<{ title: string; desc: string; milestone: string }>();
</script>

<template>
  <div class="coming">
    <h1>{{ title }}</h1>
    <p>{{ desc }}</p>
    <span class="tag">计划于 {{ milestone }} 里程碑开发</span>
  </div>
</template>

<style scoped>
.coming {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.coming h1 {
  font-size: 24px;
}

.coming p {
  color: var(--text-2);
  max-width: 520px;
  line-height: 1.8;
}

.tag {
  align-self: flex-start;
  padding: 4px 10px;
  border-radius: 12px;
  background: var(--primary-light);
  color: var(--primary);
  font-size: 12px;
}
</style>
