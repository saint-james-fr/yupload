<script setup lang="ts">
import { cn } from "../../lib/utils";

const props = withDefaults(
  defineProps<{
    label: string;
    description?: string;
    class?: string;
  }>(),
  {}
);
</script>

<template>
  <div :class="cn('', props.class)">
    <p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">{{ label }}</p>
    <p v-if="description" class="text-xs text-muted-foreground mt-0.5">{{ description }}</p>
  </div>
</template>
