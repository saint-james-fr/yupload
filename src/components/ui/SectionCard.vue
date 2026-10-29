<script setup lang="ts">
import { cn } from "../../lib/utils";

type Padding = "default" | "compact" | "none";

const props = withDefaults(
  defineProps<{
    padding?: Padding;
    class?: string;
  }>(),
  { padding: "default" }
);

const paddingClass = {
  default: "p-6",
  compact: "p-4",
  none: "",
}[props.padding];
</script>

<template>
  <div
    :class="
      cn(
        'rounded-lg border-2 border-border bg-card shadow-sm',
        paddingClass,
        props.class
      )
    "
  >
    <slot />
  </div>
</template>
