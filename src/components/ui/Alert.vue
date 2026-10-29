<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

export interface AlertProps {
  variant?: "default" | "destructive";
  class?: string;
}

const props = withDefaults(defineProps<AlertProps>(), {
  variant: "default",
});

const alertClass = computed(() =>
  cn(
    "relative w-full rounded-lg border p-4",
    props.variant === "destructive"
      ? "bg-destructive/10 text-destructive border-destructive/50"
      : "bg-background text-foreground",
    props.class
  )
);
</script>

<template>
  <div :class="alertClass" role="alert">
    <slot />
  </div>
</template>
