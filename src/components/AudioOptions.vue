<script setup lang="ts">
import Toggle from "./ui/Toggle.vue";
import SettingsRow from "./ui/SettingsRow.vue";
import SectionHeader from "./ui/SectionHeader.vue";
import { Music2, Filter } from "lucide-vue-next";

defineProps<{
  normalizeAudio: boolean;
  highPassFilter?: boolean;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:normalizeAudio", value: boolean): void;
  (e: "update:highPassFilter", value: boolean): void;
}>();
</script>

<template>
  <div class="space-y-4">
    <SectionHeader label="Audio" description="Optional loudness normalization and high-pass filter for vinyl rips." />
    <div class="rounded-lg border-2 border-border bg-card p-4 shadow-sm space-y-4">
      <!-- Normalize audio -->
      <SettingsRow
        :icon="Music2"
        title="Normalize audio"
        description="EBU R128 loudness so all tracks match."
      >
        <Toggle
          :model-value="normalizeAudio"
          :disabled="disabled"
          aria-label="Normalize audio"
          @update:model-value="emit('update:normalizeAudio', $event)"
        />
      </SettingsRow>

      <!-- 30Hz high-pass filter (when normalize is on) -->
      <div
        v-if="normalizeAudio"
        class="border-l-2 border-primary/50 rounded-r-md bg-muted/20 pl-4 py-3 -mx-1"
      >
        <SettingsRow
          :icon="Filter"
          title="30Hz high-pass filter"
          description="Removes turntable rumble. Useful for vinyl audio recordings."
        >
          <Toggle
            :model-value="highPassFilter ?? false"
            :disabled="disabled"
            aria-label="30Hz high-pass filter"
            @update:model-value="emit('update:highPassFilter', $event)"
          />
        </SettingsRow>
      </div>
    </div>
  </div>
</template>
