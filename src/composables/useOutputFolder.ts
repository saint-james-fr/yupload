import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { usePreferences } from "./usePreferences";

type PreferenceKey = "last_creation_output" | "last_normalize_output" | "last_export_output" | "last_audio_export_output" | "last_upload_source";
type DefaultPreferenceKey = "preferred_output_path" | "preferred_audio_output_path" | "preferred_video_output_path" | "preferred_audio_input_path" | "preferred_image_input_path";

export type OutputFolderType = "audio" | "video" | "general";

export function useOutputFolder(options?: {
  preferenceKey?: PreferenceKey;
  defaultPreferenceKey?: DefaultPreferenceKey;
  /** Type of output to determine which preferred path to use */
  outputType?: OutputFolderType;
}) {
  const { preferences, updatePreferences } = usePreferences();
  const outputFolder = ref<string | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const getPreferredOutputPath = (prefs: any): string | null => {
    // Determine which preferred path to use based on output type
    if (options?.outputType === "audio" && prefs?.preferred_audio_output_path) {
      return prefs.preferred_audio_output_path;
    }
    
    if (options?.outputType === "video" && prefs?.preferred_video_output_path) {
      return prefs.preferred_video_output_path;
    }
    
    // Use explicit defaultPreferenceKey if provided
    if (options?.defaultPreferenceKey && prefs?.[options.defaultPreferenceKey]) {
      return prefs[options.defaultPreferenceKey];
    }
    
    // Fallback hierarchy: type-specific → legacy preferred → null
    if (options?.outputType === "audio" && prefs?.preferred_audio_output_path) {
      return prefs.preferred_audio_output_path;
    }
    
    if (options?.outputType === "video" && prefs?.preferred_video_output_path) {
      return prefs.preferred_video_output_path;
    }
    
    // Final fallback to legacy preferred_output_path
    return prefs?.preferred_output_path ?? null;
  };

  const loadFromPreferences = async () => {
    try {
      error.value = null;
      const prefs = preferences.value.preferences;
      const lastKey = options?.preferenceKey;
      
      const last = lastKey ? (prefs?.[lastKey] ?? null) : null;
      const preferred = getPreferredOutputPath(prefs);
      const effective = last || preferred;
      
      if (effective) {
        outputFolder.value = effective;
      }
    } catch (err) {
      error.value = `Failed to load preferred output path: ${err}`;
    }
  };

  const selectFolder = async () => {
    try {
      isLoading.value = true;
      error.value = null;
      
      // Get default path from preferences using enhanced fallback logic
      const prefs = preferences.value.preferences;
      const lastKey = options?.preferenceKey;
      const last = lastKey ? (prefs?.[lastKey] ?? null) : null;
      const preferred = getPreferredOutputPath(prefs);
      const defaultPath = last || preferred || undefined;
      
      const path = await open({
        directory: true,
        multiple: false,
        defaultPath
      });
      
      if (path && typeof path === "string") {
        outputFolder.value = path;
        
        // Save to preferences if preferenceKey is provided
        if (options?.preferenceKey) {
          await updatePreferences({ [options.preferenceKey]: path } as any);
        }
      }
    } catch (err) {
      error.value = `Failed to select folder: ${err}`;
    } finally {
      isLoading.value = false;
    }
  };

  const saveToPreferences = async (path: string) => {
    try {
      if (options?.preferenceKey) {
        await updatePreferences({ [options.preferenceKey]: path } as any);
      }
    } catch (err) {
      error.value = `Failed to save output path: ${err}`;
    }
  };

  const clear = () => {
    outputFolder.value = null;
    error.value = null;
  };

  return {
    outputFolder,
    isLoading,
    error,
    loadFromPreferences,
    selectFolder,
    saveToPreferences,
    clear
  };
}
