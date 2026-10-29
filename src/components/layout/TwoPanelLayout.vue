<script setup lang="ts">
import Card from "../ui/Card.vue";

export interface TwoPanelLayoutProps {
  leftTitle?: string;
  rightTitle?: string;
  leftSubtitle?: string;
  rightSubtitle?: string;
  itemsCount?: number;
  selectedIndex: number | null;
}

defineProps<TwoPanelLayoutProps>();

defineEmits<{
  (e: "select", index: number): void;
}>();
</script>

<template>
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Left Panel: List -->
    <div class="lg:col-span-1">
      <Card class="p-4">
        <div class="space-y-2">
          <div v-if="leftTitle || itemsCount !== undefined" class="flex items-center justify-between mb-4">
            <h2 v-if="leftTitle" class="text-xl font-semibold flex items-center gap-2">
              <slot name="left-icon" />
              {{ leftTitle }}
            </h2>
            <span v-if="itemsCount !== undefined" class="text-sm text-muted-foreground">
              {{ itemsCount }}
            </span>
          </div>
          <div v-if="leftSubtitle" class="text-sm text-muted-foreground mb-4">
            {{ leftSubtitle }}
          </div>
          <slot name="left-panel" />
        </div>
      </Card>
    </div>

    <!-- Right Panel: Detail -->
    <div class="lg:col-span-2">
      <Card v-if="selectedIndex !== null" class="p-6">
        <div v-if="rightTitle || rightSubtitle" class="mb-6">
          <h2 v-if="rightTitle" class="text-2xl font-semibold mb-2">{{ rightTitle }}</h2>
          <p v-if="rightSubtitle" class="text-sm text-muted-foreground">{{ rightSubtitle }}</p>
        </div>
        <slot name="right-panel" />
      </Card>
      <Card v-else class="p-6">
        <slot name="empty-state" />
      </Card>
    </div>
  </div>
</template>
