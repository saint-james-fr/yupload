<script setup lang="ts">
import { computed, type Component } from "vue";

type SegmentedOption = {
  value: string | number;
  label: string;
  icon?: Component;
  disabled?: boolean;
};

const props = defineProps<{
  modelValue: string | number | null;
  options: SegmentedOption[];
  ariaLabel?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string | number | null): void;
}>();

const isDisabled = computed(() => props.disabled ?? false);

const isSelected = (value: string | number) => props.modelValue === value;

const selectOption = (option: SegmentedOption) => {
  if (isDisabled.value || option.disabled) return;
  if (option.value === props.modelValue) return;
  emit("update:modelValue", option.value);
};
</script>

<template>
  <div
    class="inline-flex flex-wrap gap-2 rounded-lg border border-input bg-muted/30 p-0.5 text-xs sm:text-sm"
    role="radiogroup"
    :aria-label="ariaLabel"
    :aria-disabled="isDisabled || undefined"
  >
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="inline-flex items-center gap-2 rounded-md px-3 py-1.5 font-medium transition-colors cursor-pointer"
      :class="[
        isSelected(option.value)
          ? 'bg-background text-foreground shadow-sm'
          : 'text-muted-foreground hover:bg-background/60',
        (isDisabled || option.disabled) && 'opacity-60 cursor-not-allowed hover:bg-transparent'
      ]"
      role="radio"
      :aria-checked="isSelected(option.value)"
      :disabled="isDisabled || option.disabled"
      @click="selectOption(option)"
    >
      <slot
        name="option"
        :option="option"
        :selected="isSelected(option.value)"
      >
        <component
          v-if="option.icon"
          :is="option.icon"
          class="h-4 w-4"
        />
        <span>{{ option.label }}</span>
      </slot>
    </button>
  </div>
</template>

