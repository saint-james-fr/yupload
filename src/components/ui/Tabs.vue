<script setup lang="ts">

type TabOption = {
  value: string;
  label: string;
  icon?: any;
};

const props = defineProps<{
  modelValue: string;
  options: TabOption[];
  ariaLabel?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const isSelected = (value: string) => props.modelValue === value;

const selectTab = (option: TabOption) => {
  if (option.value === props.modelValue) return;
  emit("update:modelValue", option.value);
};
</script>

<template>
  <div
    class="flex gap-1 border-b border-border mb-6"
    role="tablist"
    :aria-label="ariaLabel"
  >
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="relative px-4 py-2 text-sm font-medium transition-colors cursor-pointer border-b-2 border-transparent -mb-px"
      :class="[
        isSelected(option.value)
          ? 'text-foreground border-primary'
          : 'text-muted-foreground hover:text-foreground hover:border-muted-foreground/40'
      ]"
      role="tab"
      :aria-selected="isSelected(option.value)"
      @click="selectTab(option)"
    >
      <span class="flex items-center gap-2">
        <component
          v-if="option.icon"
          :is="option.icon"
          class="h-4 w-4"
        />
        <span>{{ option.label }}</span>
      </span>
    </button>
  </div>
</template>
