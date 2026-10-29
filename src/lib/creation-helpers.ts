import type { AudioImagePair, AudioMetadata, ImageStyle, FitMode } from "../types";
import { DEFAULT_IMAGE_STYLE, DEFAULT_FIT_MODE } from "../types";

/** Minimal preferences slice needed to build a default audio–image pair. */
export type CreationDefaultsPrefs = {
  default_image_style?: ImageStyle | null;
  default_fit_mode?: FitMode | null;
  default_background_predefined_id?: string | null;
} | null | undefined;

export const defaultPair = (
  audioPath: string,
  prefs?: CreationDefaultsPrefs
): AudioImagePair => {
  const imageStyle = prefs?.default_image_style ?? DEFAULT_IMAGE_STYLE;
  const fitMode = prefs?.default_fit_mode ?? DEFAULT_FIT_MODE;
  return {
    audioPath,
    imagePath: null,
    imageUrl: null,
    usePrevious: false,
    imageStyle,
    fitMode,
    backgroundPath: null,
    backgroundUrl: null,
    backgroundPredefinedId: prefs?.default_background_predefined_id ?? null
  };
};

export const shouldUseAlbumArt = (
  metadata: AudioMetadata | null | undefined
): boolean => {
  if (!metadata?.image_data_url) return false;
  return !!(metadata.artist || metadata.title);
};
