<script setup lang="ts">
import Card from "./ui/Card.vue";
import type { ProgressItem, ProgressItemStatus } from "../types";

interface ProgressCardProps {
  title: string;
  items?: ProgressItem[];
  preparingMessage?: string;
  showOverallProgress?: boolean;
}

withDefaults(defineProps<ProgressCardProps>(), {
  preparingMessage: "Preparing...",
  showOverallProgress: false,
});

const getStatusIcon = (status?: ProgressItemStatus) => {
  switch (status) {
    case "completed":
      return "✓";
    case "failed":
      return "✗";
    case "processing":
      return "⟳";
    case "pending":
      return "○";
    default:
      return "";
  }
};

const getStatusClass = (status?: ProgressItemStatus) => {
  switch (status) {
    case "completed":
      return "text-green-600";
    case "failed":
      return "text-red-600";
    case "processing":
      return "text-blue-600";
    case "pending":
      return "text-muted-foreground";
    default:
      return "text-muted-foreground";
  }
};
</script>

<template>
  <Card class="p-6">
    <div class="space-y-4">
      <h3 class="font-semibold">{{ title }}</h3>

      <!-- No items: just show preparing message -->
      <div v-if="!items || items.length === 0">
        <div class="text-sm text-muted-foreground">
          {{ preparingMessage }}
        </div>
      </div>

      <!-- Multiple items: status-only rows -->
      <div v-else class="space-y-3">
        <div
          v-for="(item, index) in items"
          :key="index"
          class="space-y-1.5"
        >
          <div class="flex items-center justify-between text-sm">
            <span class="font-medium">{{ item.name }}</span>
            <span v-if="item.status" :class="getStatusClass(item.status)">
              {{ getStatusIcon(item.status) }}
            </span>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ item.message }}
          </p>
          <p v-if="item.error" class="text-xs text-red-600">
            {{ item.error }}
          </p>
        </div>
      </div>

      <!-- Overall summary slot (no numeric progress) -->
      <div
        v-if="showOverallProgress && items && items.length > 0"
        class="text-xs text-muted-foreground pt-2 border-t"
      >
        <slot name="overall-summary" />
      </div>
    </div>
  </Card>
</template>
