<template>
  <Modal :open="open" @close="handleClose" class="max-w-4xl">
    <div class="space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold">Edit Image</h3>
          <p class="text-sm text-muted-foreground mt-0.5">Crop your image to the desired size and aspect ratio</p>
        </div>
        <Button variant="ghost" size="icon" @click="handleClose">
          <X class="h-4 w-4" />
        </Button>
      </div>

      <!-- Crop Tool Section -->
      <SectionCard padding="none">
        <div class="p-6 space-y-4">
          <!-- Controls -->
          <div class="flex items-center justify-between gap-4">
            <!-- Ratio Selection -->
            <div class="flex items-center gap-3">
              <Label class="text-sm font-medium">Aspect Ratio:</Label>
              <SegmentedControl v-model="selectedRatio" :options="[...IMAGE_CROP_RATIO_OPTIONS]" aria-label="Select crop aspect ratio" />
            </div>

            <!-- Zoom Controls -->
            <div class="flex items-center gap-3">
              <Label class="text-sm font-medium">Zoom:</Label>
              <div class="flex items-center gap-2">
                <Button variant="outline" size="icon" @click="adjustZoom(-0.1)" :disabled="zoom <= 0.5">
                  <ZoomOut class="h-4 w-4" />
                </Button>
                <span class="text-sm text-muted-foreground min-w-12 text-center">
                  {{ Math.round(zoom * 100) }}%
                </span>
                <Button variant="outline" size="icon" @click="adjustZoom(0.1)" :disabled="zoom >= 3">
                  <ZoomIn class="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>

          <!-- Crop Tool -->
          <div class="border border-border rounded-lg overflow-hidden bg-gray-900">
            <ImageCropTool
              ref="cropToolRef"
              :image-src="imageSrc"
              :ratio="selectedRatio"
              :zoom="zoom"
              container-class="min-h-[400px] max-h-[60vh]"
              :disabled="isProcessing"
              @crop-change="handleCropChange"
              @ready="handleCropToolReady"
            />
          </div>
        </div>
      </SectionCard>


      <!-- Actions -->
      <div class="flex items-center justify-between pt-4 border-t border-border">
        <Button variant="outline" @click="resetCrop" :disabled="isProcessing">
          <RotateCcw class="h-4 w-4 mr-2" />
          Reset
        </Button>

        <div class="flex gap-3">
          <Button variant="outline" @click="handleClose" :disabled="isProcessing"> Cancel </Button>
          <Button @click="handleSave" :disabled="!cropToolReady || isProcessing" class="min-w-[100px]">
            <Loader2 v-if="isProcessing" class="h-4 w-4 mr-2 animate-spin" />
            <Crop v-else class="h-4 w-4 mr-2" />
            {{ isProcessing ? "Processing..." : "Apply Crop" }}
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { X, ZoomIn, ZoomOut, Crop, RotateCcw, Loader2 } from "lucide-vue-next";
import Modal from "./ui/Modal.vue";
import Button from "./ui/Button.vue";
import Label from "./ui/Label.vue";
import SectionCard from "./ui/SectionCard.vue";
import SegmentedControl from "./ui/SegmentedControl.vue";
import ImageCropTool from "./ui/ImageCropTool.vue";
import { IMAGE_CROP_RATIO_OPTIONS } from "../lib/constants";
import type { CropArea, CropRatio } from "../types";

interface Props {
  open: boolean;
  imageSrc: string | null;
  initialRatio?: CropRatio;
}

interface Emits {
  (e: "close"): void;
  (e: "save", croppedImageDataUrl: string): void;
}

const props = withDefaults(defineProps<Props>(), {
  initialRatio: "1:1",
});

const emit = defineEmits<Emits>();

// Component refs
const cropToolRef = ref<InstanceType<typeof ImageCropTool> | null>(null);

// State
const selectedRatio = ref<CropRatio>(props.initialRatio);
const zoom = ref(1);
const currentCropArea = ref<CropArea>({ x: 0, y: 0, width: 200, height: 200 });
const cropToolReady = ref(false);
const isProcessing = ref(false);

// Zoom controls
const adjustZoom = (delta: number) => {
  zoom.value = Math.max(0.5, Math.min(3, zoom.value + delta));
};

// Handle crop area changes
const handleCropChange = (cropArea: CropArea) => {
  currentCropArea.value = cropArea;
};

// Handle crop tool ready state
const handleCropToolReady = (ready: boolean) => {
  cropToolReady.value = ready;
};


/** When true, cropArea is in original image pixels (from getCropInImageCoordinates). */
type CropImageOptions = { inImageCoordinates?: boolean };

// Draw cropped region to canvas and return data URL. When inImageCoordinates is true, cropArea is in image pixels.
const drawCroppedToDataUrl = (
  img: HTMLImageElement,
  cropArea: CropArea,
  _options?: CropImageOptions
): string => {
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Could not get canvas context");
  canvas.width = cropArea.width;
  canvas.height = cropArea.height;
  ctx.drawImage(
    img,
    cropArea.x,
    cropArea.y,
    cropArea.width,
    cropArea.height,
    0,
    0,
    cropArea.width,
    cropArea.height
  );
  return canvas.toDataURL("image/jpeg", 0.9);
};

// Crop image using canvas. For http(s) URLs we fetch via Tauri to avoid CORS.
const cropImage = async (
  imageSrc: string,
  cropArea: CropArea,
  options?: CropImageOptions
): Promise<string> => {
  const inImageCoordinates = options?.inImageCoordinates ?? false;

  // Fetch remote URLs through Tauri to avoid CORS (e.g. Discogs artwork)
  let src = imageSrc;
  if (src.startsWith("http://") || src.startsWith("https://")) {
    src = await invoke<string>("fetch_image_as_data_url", { url: src });
  }

  return new Promise((resolve, reject) => {
    const onImageLoaded = (img: HTMLImageElement) => {
      try {
        const source = inImageCoordinates
          ? cropArea
          : (() => {
              const displayCanvas = cropToolRef.value?.canvasRef as HTMLCanvasElement | undefined;
              if (!displayCanvas) throw new Error("Could not access crop tool canvas");
              const scaleX = img.width / displayCanvas.width;
              const scaleY = img.height / displayCanvas.height;
              return {
                x: cropArea.x * scaleX,
                y: cropArea.y * scaleY,
                width: cropArea.width * scaleX,
                height: cropArea.height * scaleY,
              };
            })();
        resolve(drawCroppedToDataUrl(img, source));
      } catch (error) {
        reject(error);
      }
    };

    const loadAsBlob = async () => {
      const tauriUrl = convertFileSrc(src);
      const response = await fetch(tauriUrl);
      const imageBlob = await response.blob();
      const blobUrl = URL.createObjectURL(imageBlob);
      const img = new Image();
      img.onload = () => {
        try {
          onImageLoaded(img);
        } finally {
          URL.revokeObjectURL(blobUrl);
        }
      };
      img.onerror = () => {
        URL.revokeObjectURL(blobUrl);
        reject(new Error("Failed to load image from blob"));
      };
      img.src = blobUrl;
    };

    try {
      if (src.startsWith("data:")) {
        const img = new Image();
        img.onload = () => onImageLoaded(img);
        img.onerror = () => reject(new Error("Failed to load data URL"));
        img.src = src;
        return;
      }
      loadAsBlob();
    } catch (error) {
      console.error("Failed to prepare image for cropping:", error);
      reject(error);
    }
  });
};

// Reset crop to initial state
const resetCrop = () => {
  cropToolRef.value?.resetCrop();
};

// Handle save action
const handleSave = async () => {
  if (!props.imageSrc || !cropToolReady.value) return;

  isProcessing.value = true;

  try {
    const imageCrop = cropToolRef.value?.getCropInImageCoordinates();
    const crop = imageCrop ?? currentCropArea.value;
    const croppedDataUrl = await cropImage(props.imageSrc, crop, {
      inImageCoordinates: !!imageCrop,
    });
    emit("save", croppedDataUrl);
  } catch (error) {
    console.error("Failed to crop image:", error);
  } finally {
    isProcessing.value = false;
  }
};

// Handle modal close
const handleClose = () => {
  if (isProcessing.value) return;
  emit("close");
};

// Watch for prop changes
watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      // Reset state when modal opens
      selectedRatio.value = props.initialRatio;
      zoom.value = 1;
      cropToolReady.value = false;
      isProcessing.value = false;
      
      // Force redraw after a short delay to ensure canvas is visible
      setTimeout(() => {
        cropToolRef.value?.forceRedraw();
      }, 100);
    }
  },
);

watch(
  () => props.initialRatio,
  (newRatio) => {
    selectedRatio.value = newRatio;
  },
);
</script>
