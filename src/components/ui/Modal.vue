<script setup lang="ts">
import { watch, onUnmounted, onMounted, onBeforeUnmount } from "vue";
import { cn } from "../../lib/utils";
import { useModalScrollLock } from "../../composables/useModalScrollLock";

export interface ModalProps {
  open: boolean;
  class?: string;
}

const props = defineProps<ModalProps>();

const emit = defineEmits<{
  close: [];
}>();

const { lock, unlock } = useModalScrollLock();

// Track if we've locked to ensure we unlock on unmount
let isLocked = false;

const stopWatch = watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      lock();
      isLocked = true;
    } else {
      unlock();
      isLocked = false;
    }
  },
  { immediate: true }
);

// Always unlock on unmount to prevent UI blocking
// This handles cases where the component unmounts while open=true
onBeforeUnmount(() => {
  stopWatch();
  if (isLocked) {
    unlock();
    isLocked = false;
  }
});

onUnmounted(() => {
  // Double-check: ensure we're unlocked when component is destroyed
  if (isLocked) {
    unlock();
    isLocked = false;
  }
});

const handleBackdropClick = (e: MouseEvent) => {
  if (e.target === e.currentTarget) {
    emit("close");
  }
};

const handleEscape = (e: KeyboardEvent) => {
  if (e.key === "Escape" && props.open) {
    emit("close");
  }
};

onMounted(() => {
  document.addEventListener("keydown", handleEscape);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleEscape);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="modal-overlay"
      @click="handleBackdropClick"
    >
      <div
        :class="cn('modal-content bg-card rounded-lg border shadow-lg p-6 max-w-md w-full mx-4', props.class)"
        @click.stop
      >
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  /* Use dynamic viewport so we center in what the user actually sees
     (e.g. excludes mobile browser chrome). Falls back to 100vh. */
  min-height: 100svh;
  min-height: 100dvh;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgb(0 0 0 / 0.5);
  /* Ensure overlay captures pointer events */
  pointer-events: auto;
}

.modal-content {
  max-height: calc(100vh - 2rem);
  max-height: calc(100dvh - 2rem);
  overflow-y: auto;
  /* Ensure content can receive pointer events */
  pointer-events: auto;
  /* Prevent content clicks from bubbling to overlay */
  position: relative;
  z-index: 1;
}
</style>
