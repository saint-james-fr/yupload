/** How to produce the 16:9 frame from the main image */
export type ImageStyle =
  | "automatic"
  | "fit16x9"
  | "custom-background"
  /** Use only the background (no centered main image). */
  | "background-only";

/** For fit16x9: how to fit the image into 16:9 */
export type FitMode = "contain" | "cover";

export const DEFAULT_IMAGE_STYLE: ImageStyle = "automatic";
export const DEFAULT_FIT_MODE: FitMode = "contain";

/** Per-track audio–image assignment (creation flow state) */
export type AudioImagePair = {
  audioPath: string;
  imagePath: string | null;
  imageUrl: string | null;
  usePrevious: boolean;
  imageStyle?: ImageStyle;
  fitMode?: FitMode;
  backgroundPath?: string | null;
  backgroundUrl?: string | null;
  backgroundPredefinedId?: string | null;
};

/** Per-video image input: main image + style + optional background */
export type ImageInput = {
  image_path?: string | null;
  image_url?: string | null;
  image_style?: ImageStyle;
  fit_mode?: FitMode;
  background_path?: string | null;
  background_url?: string | null;
  background_predefined_id?: string | null;
};

export type VideoRequest = {
  image_path: string | null;
  image_url: string | null;
  image_style?: ImageStyle;
  fit_mode?: FitMode;
  background_path?: string | null;
  background_url?: string | null;
  background_predefined_id?: string | null;
  audio_paths: string[];
  title: string;
  output_path: string;
  config: {
    width: number;
    height: number;
    fps: number;
  };
};

export type AudioMetadata = {
  artist: string | null;
  title: string | null;
  album: string | null;
  image_data_url: string | null;
};

export type FileInfo = {
  path: string;
  name: string;
  is_image: boolean;
  is_audio: boolean;
};

export type FolderContents = {
  images: FileInfo[];
  audios: FileInfo[];
  videos: FileInfo[];
};

export type BulkVideoRequest = {
  /** Per-video image configs. When present, length must match audio_paths. */
  image_inputs?: ImageInput[];
  /** Legacy: when image_inputs is absent, use these paths with automatic style. */
  image_paths?: string[];
  audio_paths: string[];
  output_path: string;
  config: {
    width: number;
    height: number;
    fps: number;
  };
  normalize_audio?: boolean;
  high_pass_filter?: boolean;
  export_normalized_audio?: boolean;
  audio_export_path?: string;
};

export type BulkVideoResult = {
  successful: string[];
  failed: Array<[string, string]>;
  exported_audio_files?: string[];
};

export type VideoUploadResult = {
  video_id: string;
  video_url: string;
};

export type BulkVideoUploadResult = {
  successful: VideoUploadResult[];
  failed: Array<[string, string]>;
};

export type OAuthConfig = {
  client_id: string;
  client_secret: string;
  redirect_uri: string;
};

export type DiscogsFormat = {
  name: string;
  qty?: string | null;
  descriptions: string[];
};

export type DiscogsImage = {
  uri: string;
  uri150?: string | null;
  type?: string | null;
};

export type DiscogsRelease = {
  id: number;
  title: string;
  artists: DiscogsArtist[];
  year: number | null;
  labels: DiscogsLabel[];
  genres: string[];
  styles: string[];
  tracklist: DiscogsTrack[];
  credits: DiscogsCredit[];
  extraartists: DiscogsArtist[];
  master_id: number | null;
  master_url: string | null;
  format: string[] | null;
  formats: DiscogsFormat[];
  country: string | null;
  notes: string | null;
  uri: string | null;
  /** Optional images from the release payload (used for artwork selection). */
  images?: DiscogsImage[];
};

export type DiscogsArtist = {
  name: string;
  anv: string | null;
  join: string | null;
  role: string | null;
  tracks: string | null;
  id: number | null;
};

export type DiscogsLabel = {
  name: string;
  catno: string | null;
  entity_type: string | null;
  entity_type_name: string | null;
  id: number | null;
};

export type DiscogsTrack = {
  position: string | null;
  title: string;
  duration: string | null;
  type_: string | null;
};

export type DiscogsCredit = {
  name: string;
  role: string;
  id: number | null;
};

export type DiscogsSearchResult = {
  id: number;
  title: string;
  year: number | null;
  thumb: string | null;
  cover_image: string | null;
  resource_url: string | null;
  master_id: number | null;
  master_url: string | null;
  uri: string | null;
  country: string | null;
  format: string[] | null;
  label: string[] | null;
  catno: string | null;
  barcode: string[]; // Always present, can be empty array
  community: DiscogsCommunity | null;
  type_: string | null;
  genre: string[] | null;
  style: string[] | null;
  user_data: DiscogsUserData | null;
};

export type DiscogsUserData = {
  in_wantlist: boolean | null;
  in_collection: boolean | null;
};

export type DiscogsCommunity = {
  want: number | null;
  have: number | null;
};

export type DiscogsSearchResponse = {
  pagination: DiscogsPagination;
  results: DiscogsSearchResult[];
};

export type DiscogsPagination = {
  page: number;
  pages: number;
  per_page: number;
  items: number;
  urls: DiscogsPaginationUrls | null;
};

export type DiscogsPaginationUrls = {
  next: string | null;
  prev: string | null;
};

export type VideoProcessingStatus = 
  | "uploaded" 
  | "processing" 
  | "processed" 
  | "failed" 
  | "deleted";

/** Per-video metadata for upload flows (used by metadata editing layout / future useVideoUploadFlow). */
export type VideoMetadataItem = {
  title: string;
  description: string;
  tags: string[];
  categoryId: string;
  privacyStatus: "public" | "private" | "unlisted" | "draft";
  selfDeclaredMadeForKids: boolean;
  publishAt: string | null;
  playlistId: string | null;
  discogsReleaseId: number | null;
  discogsRelease: DiscogsRelease | null;
  videoPath?: string;
};

/** Upload-only flow: video with required path. */
export type VideoUploadMetadata = VideoMetadataItem & { videoPath: string };

export type UploadHistoryEntry = {
  id: string;
  title: string;
  youtubeVideoId: string;
  youtubeUrl: string;
  discogsReleaseId: number | null;
  discogsReleaseUrl: string | null;
  discogsReleaseTitle: string | null;
  uploadedAt: string; // ISO 8601 date string
  // Processing status fields (optional for backward compatibility)
  processingStatus?: VideoProcessingStatus;
  processingStatusCheckedAt?: string; // ISO 8601 timestamp
  processingFailureReason?: string | null;
};

// Description template builder (upload description prefs)
export type DescriptionTagKey =
  | "artist"
  | "release"
  | "year"
  | "label"
  | "catalogNumber"
  | "genres"
  | "styles"
  | "discogsUrl";

export type DescriptionTemplateBlockType = "tag" | "text";

export type DescriptionTemplateBlock = {
  id: string;
  type: DescriptionTemplateBlockType;
  enabled: boolean;
  tagKey?: DescriptionTagKey;
  content?: string;
};

export type DescriptionTemplate = DescriptionTemplateBlock[];

// Progress tracking types
export type ProgressItemStatus = "pending" | "processing" | "completed" | "failed";

export type ProgressItem = {
  name: string;
  /** Optional short status/step message for this item. */
  message: string;
  status?: ProgressItemStatus;
  error?: string;
  progress?: number;
};

// Pipeline progress types
export type StepStatus = "pending" | "active" | "done" | "failed" | "cancelled";

export type PipelinePhase = {
  id: string;
  label: string;
  description?: string;
  status: StepStatus;
  /** Optional message shown under the phase label. */
  message?: string;
  /** Optional aggregate counts (e.g. videos completed / total). */
  total?: number;
  completed?: number;
  /** Optional icon component (e.g. LucideIcon). */
  icon?: any;
};

export type PipelineItem = {
  id: string | number;
  name: string;
  status: StepStatus;
  /** Short label for the current step, e.g. "Processing image", "Encoding video". */
  stepLabel?: string;
  /** Optional detailed message (shown under the item). */
  message?: string;
  error?: string;
  progress?: number;
  /** Optional thumbnail data URL for the item (e.g. video frame). Shown as small inline thumbnail in pipeline. */
  thumbnailUrl?: string;
};

// Image editing types
export type CropRatio = "1:1" | "16:9" | "4:3" | "free";

export type CropArea = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type ImageEditingState = {
  originalImage: HTMLImageElement | null;
  cropArea: CropArea;
  ratio: CropRatio;
  zoom: number;
  canvasWidth: number;
  canvasHeight: number;
  imageScale: number;
  imageOffsetX: number;
  imageOffsetY: number;
};
