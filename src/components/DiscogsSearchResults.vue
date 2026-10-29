<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import type { DiscogsSearchResult } from "../types";
import { Music } from "lucide-vue-next";

const props = defineProps<{
  results: DiscogsSearchResult[];
  isDisabled?: boolean;
  isLoading?: boolean;
}>();

const emit = defineEmits<{
  (e: "select", result: DiscogsSearchResult): void;
}>();

const resultsContainerRef = ref<HTMLElement | null>(null);

// Scroll to top when results change
watch(
  () => props.results,
  (newResults) => {
    // Only scroll if we have new results and the container exists
    if (newResults.length > 0 && resultsContainerRef.value) {
      // Use nextTick to ensure DOM is updated
      nextTick(() => {
        if (resultsContainerRef.value) {
          resultsContainerRef.value.scrollTop = 0;
        }
      });
    }
  },
  { deep: true }
);
</script>

<template>
  <div
    v-if="results.length > 0"
    ref="resultsContainerRef"
    class="max-h-56 overflow-y-auto border rounded-md bg-background shadow-sm"
  >
    <button
      v-for="result in results"
      :key="result.id"
      type="button"
      @click="emit('select', result)"
      class="w-full text-left p-3 hover:bg-accent border-b last:border-b-0 flex items-start gap-3 text-xs transition-colors cursor-pointer"
      :disabled="isDisabled || isLoading"
    >
      <img
        v-if="result.thumb"
        :src="result.thumb"
        :alt="result.title"
        class="w-12 h-12 md:w-16 md:h-16 object-cover rounded shrink-0"
      />
      <div v-else class="w-12 h-12 md:w-16 md:h-16 rounded bg-muted shrink-0 flex items-center justify-center">
        <Music class="h-4 w-4 md:h-6 md:w-6 text-muted-foreground" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="font-medium truncate">{{ result.title }}</p>
        <div class="flex flex-wrap gap-1 mt-0.5">
          <span
            v-if="result.year"
            class="inline-flex items-center px-1.5 py-0.5 rounded bg-muted text-[10px] text-muted-foreground"
          >
            {{ result.year }}
          </span>
          <span
            v-if="result.country"
            class="inline-flex items-center px-1.5 py-0.5 rounded bg-muted text-[10px] text-muted-foreground"
          >
            {{ result.country }}
          </span>
          <span
            v-if="result.format && result.format.length > 0"
            class="inline-flex items-center px-1.5 py-0.5 rounded bg-muted text-[10px] text-muted-foreground"
          >
            {{ result.format.join(", ") }}
          </span>
        </div>
        <p
          v-if="result.label && result.label.length > 0"
          class="text-[11px] text-muted-foreground truncate mt-0.5"
        >
          {{ result.label.join(", ") }}
        </p>
        <p
          v-if="result.genre && result.genre.length > 0"
          class="text-[11px] text-muted-foreground truncate"
        >
          {{ result.genre.join(", ") }}
        </p>
      </div>
    </button>
  </div>
</template>

