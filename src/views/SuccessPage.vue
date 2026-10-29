<script setup lang="ts">
import { computed } from "vue";
import PageContainer from "../components/layout/PageContainer.vue";
import Button from "../components/ui/Button.vue";
import { CheckCircle2, Home, History, ArrowLeft } from "lucide-vue-next";

export type SuccessFrom = "upload" | "unified-upload" | "normalize";

export type SuccessContext = {
  from: SuccessFrom;
  title: string;
  summary: string;
};

const props = withDefaults(
  defineProps<{
    title: string;
    summary: string;
    from: SuccessFrom;
  }>(),
  {}
);

const emit = defineEmits<{
  (e: "navigate", view: "menu" | "upload" | "unified-upload" | "normalize" | "history"): void;
}>();

const backToLabel = computed(() => {
  const labels: Record<SuccessFrom, string> = {
    upload: "Video Upload",
    "unified-upload": "Create & Upload",
    normalize: "Audio Normalization",
  };
  return labels[props.from];
});

const backToView = computed((): "upload" | "unified-upload" | "normalize" => {
  if (props.from === "unified-upload") return "unified-upload";
  return props.from;
});

const goHome = () => emit("navigate", "menu");
const goBackToFlow = () => emit("navigate", backToView.value);
const goToHistory = () => emit("navigate", "history");
</script>

<template>
  <PageContainer class="flex flex-col items-center justify-center min-h-screen" max-width="xl">
    <div class="w-full max-w-md space-y-8 text-center">
      <div class="flex justify-center">
        <div
          class="flex h-20 w-20 items-center justify-center rounded-full bg-green-500/10 border-2 border-green-500/30"
        >
          <CheckCircle2 class="h-12 w-12 text-green-600 dark:text-green-400" stroke-width="1.5" />
        </div>
      </div>
      <div class="space-y-2">
        <h1 class="text-2xl font-semibold text-foreground">{{ title }}</h1>
        <p class="text-sm text-muted-foreground">{{ summary }}</p>
      </div>
      <div class="flex flex-col gap-3">
        <Button class="w-full justify-center gap-2" @click="goHome">
          <Home class="h-4 w-4" />
          Back to home
        </Button>
        <Button variant="outline" class="w-full justify-center gap-2" @click="goBackToFlow">
          <ArrowLeft class="h-4 w-4" />
          Back to {{ backToLabel }}
        </Button>
        <Button
          v-if="from === 'upload' || from === 'unified-upload'"
          variant="outline"
          class="w-full justify-center gap-2"
          @click="goToHistory"
        >
          <History class="h-4 w-4" />
          See upload history
        </Button>
      </div>
    </div>
  </PageContainer>
</template>
