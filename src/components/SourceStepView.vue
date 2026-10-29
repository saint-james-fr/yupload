<script setup lang="ts">
import PageContainer from "./layout/PageContainer.vue";
import PageHeader from "./layout/PageHeader.vue";

defineProps<{
  title?: string;
  subtitle?: string;
}>();

const emit = defineEmits<{
  (e: "back"): void;
}>();

const handleBack = () => {
  emit("back");
};
</script>

<template>
  <PageContainer>
    <PageHeader
      :title="title ?? 'Select source'"
      :description="subtitle"
      :show-back-button="true"
      @back="handleBack"
    />
    <div class="source-step-content">
      <slot />
    </div>
  </PageContainer>
</template>

<style scoped>
.source-step-content {
  width: 100%;
  max-width: min(36rem, 90vw);
  margin-left: auto;
  margin-right: auto;
  padding-top: 0.5rem;
}
</style>
