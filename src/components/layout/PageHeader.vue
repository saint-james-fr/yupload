<script setup lang="ts">
import { computed } from "vue";
import Button from "../ui/Button.vue";
import { ChevronRight } from "lucide-vue-next";
import Card from "../ui/Card.vue";

export interface PageHeaderProps {
  title: string;
  description?: string;
  showBackButton?: boolean;
}

const props = withDefaults(defineProps<PageHeaderProps>(), {
  showBackButton: true,
});

const emit = defineEmits<{
  (e: "back"): void;
}>();

const handleBack = () => {
  emit("back");
};

// Split description into paragraphs (by periods followed by space and capital letter)
const descriptionParagraphs = computed(() => {
  if (!props.description) return [];
  
  // Split by periods followed by space and capital letter (new sentence)
  const sentences = props.description.split(/(?<=\.)\s+(?=[A-Z])/);
  
  // Group sentences into paragraphs (every 2-3 sentences)
  const paragraphs: string[] = [];
  let currentParagraph: string[] = [];
  
  sentences.forEach((sentence, index) => {
    const trimmed = sentence.trim();
    if (!trimmed) return;
    
    currentParagraph.push(trimmed);
    
    // Create a new paragraph every 2 sentences, or if it's the last sentence
    if (currentParagraph.length >= 2 || index === sentences.length - 1) {
      paragraphs.push(currentParagraph.join(" "));
      currentParagraph = [];
    }
  });
  
  // If we have leftover sentences, add them as a final paragraph
  if (currentParagraph.length > 0) {
    paragraphs.push(currentParagraph.join(" "));
  }
  
  return paragraphs.length > 0 ? paragraphs : [props.description];
});
</script>

<template>
  <div class="space-y-4">
    <!-- Title and Actions Row -->
    <div class="flex items-start justify-between gap-4">
      <div class="flex-1 min-w-0">
        <h1 class="text-4xl font-bold mb-3">{{ title }}</h1>
      </div>
      <div class="flex items-center gap-2 shrink-0 flex-wrap">
        <slot name="actions" />
        <Button
          v-if="showBackButton"
          @click="handleBack"
          variant="ghost"
          size="sm"
          class="shrink-0 min-w-38"
        >
          <ChevronRight class="h-4 w-4 mr-2 rotate-180" />
          Back to Menu
        </Button>
      </div>
    </div>

    <!-- Description Card -->
    <Card v-if="description" class="p-5 bg-muted/40 border-muted/50">
      <div class="space-y-3">
        <p
          v-for="(paragraph, index) in descriptionParagraphs"
          :key="index"
          class="text-sm leading-relaxed text-foreground/85"
        >
          {{ paragraph }}
        </p>
      </div>
    </Card>
  </div>
</template>
