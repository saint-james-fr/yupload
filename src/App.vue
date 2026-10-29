<script setup lang="ts">
import { ref, onMounted } from "vue";
import MainMenu from "./views/MainMenu.vue";
import VideoUpload from "./views/VideoUpload.vue";
import UnifiedUploadPage from "./views/UnifiedUploadPage.vue";
import Settings from "./views/Settings.vue";
import AudioNormalize from "./views/AudioNormalize.vue";
import UploadHistory from "./views/UploadHistory.vue";
import FAQ from "./views/FAQ.vue";
import PrivacyPolicy from "./views/PrivacyPolicy.vue";
import SuccessPage from "./views/SuccessPage.vue";
import type { SuccessContext } from "./views/SuccessPage.vue";
import { usePredefinedBackgrounds } from "./composables/usePredefinedBackgrounds";
import { usePreferences } from "./composables/usePreferences";

type View = "menu" | "creation" | "upload" | "unified-upload" | "settings" | "normalize" | "history" | "faq" | "privacy" | "success";

const currentView = ref<View>("menu");
const successContext = ref<SuccessContext | null>(null);

const handleViewSelect = (
  view: "upload" | "unified-upload" | "settings" | "normalize" | "history" | "faq" | "privacy" | "success",
  payload?: SuccessContext
) => {
  if (view === "success" && payload) {
    successContext.value = payload;
    currentView.value = "success";
  } else {
    successContext.value = null;
    currentView.value = view === "success" ? "menu" : view;
  }
};

const handleBackToMenu = () => {
  currentView.value = "menu";
  successContext.value = null;
};

const handleSuccessNavigate = (
  view: "menu" | "upload" | "unified-upload" | "normalize" | "history"
) => {
  successContext.value = null;
  currentView.value = view;
};

// Preload predefined backgrounds at app launch
const { loadPreferences } = usePreferences();
const { loadPredefinedBackgrounds } = usePredefinedBackgrounds();
onMounted(async () => {
  // Load preferences first to ensure validation works correctly
  await loadPreferences();
  // Then load predefined backgrounds
  loadPredefinedBackgrounds();
});
</script>

<template>
  <MainMenu v-if="currentView === 'menu'" @select="handleViewSelect" />
  <VideoUpload
    v-else-if="currentView === 'upload'"
    @back="handleBackToMenu"
    @navigate="handleViewSelect"
  />
  <UnifiedUploadPage
    v-else-if="currentView === 'unified-upload'"
    @back="handleBackToMenu"
    @navigate="handleViewSelect"
  />
  <Settings v-else-if="currentView === 'settings'" @back="handleBackToMenu" />
  <AudioNormalize
    v-else-if="currentView === 'normalize'"
    @back="handleBackToMenu"
    @navigate="handleViewSelect"
  />
  <UploadHistory v-else-if="currentView === 'history'" @back="handleBackToMenu" />
  <FAQ v-else-if="currentView === 'faq'" @back="handleBackToMenu" />
  <PrivacyPolicy v-else-if="currentView === 'privacy'" @back="handleBackToMenu" />
  <SuccessPage
    v-else-if="currentView === 'success' && successContext"
    :title="successContext.title"
    :summary="successContext.summary"
    :from="successContext.from"
    @navigate="handleSuccessNavigate"
  />
</template>
