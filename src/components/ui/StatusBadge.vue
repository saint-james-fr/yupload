<script setup lang="ts">
import { computed } from "vue";
import { Lock, Unlock } from "lucide-vue-next";
import { cn } from "../../lib/utils";

const props = withDefaults(
  defineProps<{
    connected: boolean;
    connectedLabel?: string;
    disconnectedLabel?: string;
    class?: string;
  }>(),
  {
    connectedLabel: "Connected",
    disconnectedLabel: "Not connected",
  }
);

const badgeClass = computed(() =>
  cn(
    "inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium border-2 shrink-0",
    props.connected
      ? "border-green-500/50 bg-green-500/10 text-green-700 dark:text-green-400"
      : "border-muted-foreground/30 bg-muted/30 text-muted-foreground",
    props.class
  )
);
</script>

<template>
  <span :class="badgeClass" role="status">
    <Unlock v-if="connected" class="h-3.5 w-3.5" aria-hidden="true" />
    <Lock v-else class="h-3.5 w-3.5" aria-hidden="true" />
    {{ connected ? connectedLabel : disconnectedLabel }}
  </span>
</template>
