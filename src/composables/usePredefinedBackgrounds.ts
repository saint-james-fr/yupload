import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePreferences } from "./usePreferences";

type Background = {
  id: string;
  label: string;
  imagePath: string;
};

const predefinedBackgrounds = ref<Background[]>([]);
const isLoadingPredefinedBackgrounds = ref(false);
const predefinedBackgroundsLoadError = ref<string | null>(null);

const userBackgrounds = ref<Background[]>([]);
const isLoadingUserBackgrounds = ref(false);
const userBackgroundsLoadError = ref<string | null>(null);

let loadedPredefinedOnce = false;
let loadedUserOnce = false;

export const usePredefinedBackgrounds = () => {
  const { preferences, updatePreferences } = usePreferences();

  const loadPredefinedBackgrounds = async () => {
    if (loadedPredefinedOnce && predefinedBackgrounds.value.length > 0) return;
    try {
      isLoadingPredefinedBackgrounds.value = true;
      predefinedBackgroundsLoadError.value = null;
      const list = await invoke<Array<[string, string, string]>>("list_predefined_backgrounds");
      const backgrounds = list.map(([id, label, imagePath]) => ({ id, label, imagePath }));

      // Validate default_background_predefined_id preference against current list.
      const prefs = preferences.value.preferences;
      const defaultBgId = prefs?.default_background_predefined_id ?? null;
      if (defaultBgId) {
        const exists = backgrounds.some(bg => bg.id === defaultBgId);
        if (!exists) {
          // Clear invalid preference once so we don't keep trying.
          updatePreferences({ default_background_predefined_id: null });
        } else {
          // Sort backgrounds so the default one appears first
          backgrounds.sort((a, b) => {
            if (a.id === defaultBgId) return -1;
            if (b.id === defaultBgId) return 1;
            return 0;
          });
        }
      }
      predefinedBackgrounds.value = backgrounds;
      loadedPredefinedOnce = true;
    } catch (err) {
      predefinedBackgroundsLoadError.value = "Could not load templates.";
      predefinedBackgrounds.value = [];
    } finally {
      isLoadingPredefinedBackgrounds.value = false;
    }
  };

  const loadUserBackgrounds = async () => {
    if (loadedUserOnce && userBackgrounds.value.length > 0) return;
    try {
      isLoadingUserBackgrounds.value = true;
      userBackgroundsLoadError.value = null;
      const list = await invoke<Array<[string, string, string]>>("list_user_backgrounds");
      const backgrounds = list.map(([id, label, imagePath]) => ({ id, label, imagePath }));

      // Validate default_background_predefined_id preference against user backgrounds too
      const prefs = preferences.value.preferences;
      const defaultBgId = prefs?.default_background_predefined_id ?? null;
      if (defaultBgId) {
        const exists = backgrounds.some(bg => bg.id === defaultBgId);
        if (!exists) {
          // Only clear if predefined backgrounds also don't have it
          const predefExists = predefinedBackgrounds.value.some(bg => bg.id === defaultBgId);
          if (!predefExists) {
            updatePreferences({ default_background_predefined_id: null });
          }
        }
      }
      userBackgrounds.value = backgrounds;
      loadedUserOnce = true;
    } catch (err) {
      userBackgroundsLoadError.value = "Could not load user backgrounds.";
      userBackgrounds.value = [];
    } finally {
      isLoadingUserBackgrounds.value = false;
    }
  };

  const reloadUserBackgrounds = async () => {
    loadedUserOnce = false;
    await loadUserBackgrounds();
  };

  return {
    predefinedBackgrounds,
    isLoadingPredefinedBackgrounds,
    predefinedBackgroundsLoadError,
    userBackgrounds,
    isLoadingUserBackgrounds,
    userBackgroundsLoadError,
    loadPredefinedBackgrounds,
    loadUserBackgrounds,
    reloadUserBackgrounds
  };
};
