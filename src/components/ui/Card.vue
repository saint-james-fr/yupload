<script setup lang="ts">
import { computed } from "vue";
import { cn } from "../../lib/utils";

export interface CardProps {
  class?: string;
}

const props = defineProps<CardProps>();

const cardClass = computed(() =>
  cn(
    "rounded-lg border bg-card text-card-foreground shadow-sm transition-all duration-300",
    "hover:shadow-lg hover:shadow-primary/10",
    "dark:hover:shadow-[var(--color-primary-accent)]/20",
    props.class
  )
);
</script>

<template>
  <div :class="cardClass">
    <slot />
  </div>
</template>
