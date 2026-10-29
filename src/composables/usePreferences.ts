import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageStyle, FitMode, DescriptionTemplate } from "../types";

export type AppPreferences = {
  // Legacy single output path (maintained for backward compatibility)
  preferred_output_path?: string | null;

  // Enhanced output paths - separate for audio and video
  preferred_audio_output_path?: string | null;
  preferred_video_output_path?: string | null;

  // Predefined input paths for file selection
  preferred_audio_input_path?: string | null;
  preferred_image_input_path?: string | null;

  // Creation defaults
  default_normalize_for_creation?: boolean | null;
  default_high_pass_when_normalizing?: boolean | null;
  default_image_style?: ImageStyle | null;
  default_fit_mode?: FitMode | null;
  default_background_predefined_id?: string | null;

  // Upload defaults
  default_privacy_status?: "public" | "private" | "unlisted" | "draft" | null;
  default_category_id?: string | null;
  default_playlist_id?: string | null;

  // Notifications
  notify_on_upload_complete?: boolean | null;

  // Audio normalization output behaviour
  // When true or null: add "-normalized" suffix (recommended).
  // When false: overwrite originals in the chosen output folder.
  normalize_audio_add_suffix?: boolean | null;

  // Last-used paths per flow
  last_creation_output?: string | null;
  last_normalize_output?: string | null;
  last_export_output?: string | null;
  last_audio_export_output?: string | null;
  last_upload_source?: string | null;

  // Description template (blocks of tags + free text)
  description_template_blocks?: DescriptionTemplate | null;

  // Unified upload defaults
  default_save_videos_locally?: boolean | null;
  default_upload_to_youtube?: boolean | null;
};

type PreferencesState = {
  preferences: AppPreferences | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
};

const state = ref<PreferencesState>({
  preferences: null,
  isLoading: false,
  isSaving: false,
  error: null
});

let loadedOnce = false;

// Default description template blocks (same as in Settings.vue)
const getDefaultDescriptionTemplate = (): DescriptionTemplate => {
  const now = Date.now();
  return [
    {
      id: `artists-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "artist",
      content: undefined
    },
    {
      id: `release-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "release",
      content: undefined
    },
    {
      id: `label-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "label",
      content: undefined
    },
    {
      id: `catalogNumber-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "catalogNumber",
      content: undefined
    },
    {
      id: `genres-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "genres",
      content: undefined
    },
    {
      id: `styles-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "styles",
      content: undefined
    },
    {
      id: `year-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "year",
      content: undefined
    },
    {
      id: `divider-${now}`,
      type: "text",
      enabled: true,
      tagKey: undefined,
      content: ""
    },
    {
      id: `discogs-${now}`,
      type: "tag",
      enabled: true,
      tagKey: "discogsUrl",
      content: undefined
    }
  ];
};

export const usePreferences = () => {
  const loadPreferences = async () => {
    if (loadedOnce && state.value.preferences) return;
    try {
      state.value.isLoading = true;
      state.value.error = null;
      const raw = await invoke<AppPreferences>("get_app_preferences");

      // Coerce any invalid values back to safe defaults on the frontend only.
      const normalized: AppPreferences = { ...raw };
      
      // Initialize default description template if missing
      if (!normalized.description_template_blocks || normalized.description_template_blocks.length === 0) {
        normalized.description_template_blocks = getDefaultDescriptionTemplate();
        // Save the initialized template back to the backend
        await invoke("set_app_preferences", { prefs: normalized });
      }

      const validImageStyles: ImageStyle[] = ["automatic", "fit16x9", "custom-background", "background-only"]
      if (normalized.default_image_style && !validImageStyles.includes(normalized.default_image_style)) {
        normalized.default_image_style = "automatic";
      }

      const validFitModes: FitMode[] = ["contain", "cover"];
      if (normalized.default_fit_mode && !validFitModes.includes(normalized.default_fit_mode)) {
        normalized.default_fit_mode = "contain";
      }

      const validPrivacy: AppPreferences["default_privacy_status"][] = [
        "public",
        "private",
        "unlisted",
        "draft"
      ];
      if (normalized.default_privacy_status && !validPrivacy.includes(normalized.default_privacy_status)) {
        normalized.default_privacy_status = "private";
      }

      state.value.preferences = normalized;
      loadedOnce = true;
    } catch (err) {
      console.error("Failed to load preferences:", err);
      state.value.error = String(err);
    } finally {
      state.value.isLoading = false;
    }
  };

  const updatePreferences = async (patch: Partial<AppPreferences>) => {
    try {
      state.value.isSaving = true;
      state.value.error = null;
      const current: AppPreferences = state.value.preferences ?? {};
      const merged: AppPreferences = {
        ...current,
        ...patch
      };
      await invoke("set_app_preferences", { prefs: merged });
      state.value.preferences = merged;
    } catch (err) {
      console.error("Failed to save preferences:", err);
      state.value.error = String(err);
    } finally {
      state.value.isSaving = false;
    }
  };

  return {
    preferences: state,
    loadPreferences,
    updatePreferences
  };
};

