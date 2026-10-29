<script setup lang="ts">
import Button from "./Button.vue";
import { ChevronRight } from "lucide-vue-next";

const emit = defineEmits<{
  (e: "back-to-menu"): void;
}>();
</script>

<template>
  <div class="mt-8 pt-6 border-t border-border">
    <div class="flex justify-center">
      <Button
        variant="ghost"
        size="sm"
        @click="emit('back-to-menu')"
        class="shrink-0 min-w-38"
      >
        <ChevronRight class="h-4 w-4 mr-2 rotate-180" />
        Back to Menu
      </Button>
    </div>
  </div>
</template>