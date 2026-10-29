<script setup lang="ts">
import { ref, watch } from "vue";
import { ChevronDown } from "lucide-vue-next";
import { cn } from "../../lib/utils";

const props = withDefaults(
  defineProps<{
    /** Trigger label (e.g. "Technical details") */
    label: string;
    /** Start expanded */
    defaultOpen?: boolean;
    /** Optional unique id for aria-controls (needed when multiple on same page) */
    bodyId?: string;
    class?: string;
  }>(),
  { defaultOpen: false }
);

const isOpen = ref(props.defaultOpen);
const contentId = props.bodyId ? `collapsible-${props.bodyId}` : "collapsible-body";

watch(
  () => props.defaultOpen,
  (v) => {
    isOpen.value = v;
  },
  { immediate: true }
);

const toggle = () => {
  isOpen.value = !isOpen.value;
};
</script>

<template>
  <div
    :class="cn('rounded-lg border-2 border-border bg-card overflow-hidden', props.class)"
    role="region"
    :aria-expanded="isOpen"
  >
    <button
      type="button"
      class="w-full flex items-center justify-between gap-2 px-4 py-3 text-left text-sm font-medium text-foreground hover:bg-muted/40 transition-colors"
      :aria-expanded="isOpen"
      :aria-controls="contentId"
      @click="toggle"
    >
      <span>{{ label }}</span>
      <ChevronDown
        :class="cn('h-4 w-4 shrink-0 text-muted-foreground transition-transform', isOpen && 'rotate-180')"
        aria-hidden
      />
    </button>
    <div
      :id="contentId"
      class="border-t border-border"
      role="region"
      :aria-label="label"
    >
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        enter-from-class="opacity-0 -translate-y-1"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition-all duration-150 ease-in"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-1"
      >
        <div v-show="isOpen" class="px-4 py-3 bg-muted/20">
          <div class="text-sm text-muted-foreground leading-relaxed space-y-2">
            <slot />
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>
