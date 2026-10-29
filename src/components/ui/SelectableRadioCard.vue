<script setup lang="ts">
import { computed, type Component, useSlots } from "vue";
import { cn } from "../../lib/utils";

export interface SelectableRadioCardProps {
  value: string | number;
  modelValue: string | number | null;
  icon?: Component;
  title: string;
  description?: string;
  disabled?: boolean;
  /** Optional slot for custom content (e.g., SVG illustrations) */
  class?: string;
}

const props = withDefaults(defineProps<SelectableRadioCardProps>(), {
  disabled: false,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string | number): void;
}>();

const slots = useSlots();

const isSelected = computed(() => props.modelValue === props.value);

const handleChange = () => {
  if (props.disabled) return;
  emit("update:modelValue", props.value);
};

const isFlexCol = computed(() => props.class?.includes('flex-col') ?? false);
const hasCustomContent = computed(() => !!slots.content);

const cardClass = computed(() =>
  cn(
    "selectable-radio-card flex rounded-lg border-2 cursor-pointer transition-all p-3 relative",
    isFlexCol.value && hasCustomContent.value 
      ? "items-center justify-center" 
      : "items-start gap-3",
    isSelected.value
      ? "is-selected border-primary shadow-md shadow-primary/20"
      : "border-border/50",
    props.disabled && "opacity-50 cursor-not-allowed",
    props.class
  )
);
</script>

<template>
  <label :class="cardClass">
    <input
      type="radio"
      class="sr-only"
      :checked="isSelected"
      :disabled="disabled"
      @change="handleChange"
    />
    <div
      v-if="icon"
      class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-md"
      style="background: hsl(var(--muted));"
    >
      <component :is="icon" class="h-4 w-4 text-muted-foreground" />
    </div>
    <div :class="cn('min-w-0', isFlexCol && hasCustomContent ? 'w-full flex flex-col items-center' : 'flex-1')">
      <slot name="content">
        <span :class="cn('text-sm font-semibold block', isSelected ? 'text-foreground' : 'text-foreground/90')">{{ title }}</span>
        <p v-if="description" :class="cn('text-xs mt-1 leading-relaxed', isSelected ? 'text-muted-foreground' : 'text-muted-foreground/70')">
          {{ description }}
        </p>
      </slot>
    </div>
  </label>
</template>

<style scoped>
.selectable-radio-card > * {
  position: relative;
}
</style>
