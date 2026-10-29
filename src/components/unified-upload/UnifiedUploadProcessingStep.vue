<script setup lang="ts">
import type { PipelinePhase, PipelineItem } from "../../types";
import PipelineProgress from "../PipelineProgress.vue";

defineProps<{
  phases: PipelinePhase[];
  itemsByPhase: Record<string, PipelineItem[]>;
}>();

const emit = defineEmits<{
  (e: "cancel-phase", phaseId: string): void;
  (e: "cancel-item", phaseId: string, itemId: string | number): void;
}>();

const onCancelPhase = (phaseId: string) => emit("cancel-phase", phaseId);
const onCancelItem = (phaseId: string, itemId: string | number) => emit("cancel-item", phaseId, itemId);
</script>

<template>
  <div class="space-y-4">
    <PipelineProgress
      title="Processing pipeline"
      :phases="phases"
      :items-by-phase="itemsByPhase"
      :show-item-progress="true"
      :on-cancel-phase="onCancelPhase"
      :on-cancel-item="onCancelItem"
    />
  </div>
</template>
