<template>
  <div class="relative bg-gray-900 rounded-lg overflow-hidden flex items-center justify-center" :class="containerClass">
    <canvas
      ref="canvasRef"
      :width="canvasWidth"
      :height="canvasHeight"
      class="block max-w-full max-h-full cursor-crosshair"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
    />
    
    <!-- Loading overlay -->
    <div
      v-if="isLoading"
      class="absolute inset-0 bg-background/80 flex items-center justify-center"
    >
      <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Loader2 } from 'lucide-vue-next'
import type { CropArea, CropRatio } from '../../types'

interface Props {
  imageSrc: string | null
  ratio: CropRatio
  zoom: number
  containerClass?: string
  disabled?: boolean
}

interface Emits {
  (e: 'crop-change', cropArea: CropArea): void
  (e: 'ready', ready: boolean): void
}

const props = withDefaults(defineProps<Props>(), {
  ratio: 'free',
  zoom: 1,
  containerClass: '',
  disabled: false
})

const emit = defineEmits<Emits>()

// Canvas and image state
const canvasRef = ref<HTMLCanvasElement | null>(null)
const canvasWidth = ref(800)
const canvasHeight = ref(600)
const isLoading = ref(false)

// Image state
const originalImage = ref<HTMLImageElement | null>(null)
const imageScale = ref(1)
const imageOffsetX = ref(0)
const imageOffsetY = ref(0)

// Crop area state
const cropArea = ref<CropArea>({
  x: 0,
  y: 0,
  width: 200,
  height: 200
})

// Interaction state
const isDragging = ref(false)
const isResizing = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartCrop = ref<CropArea>({ x: 0, y: 0, width: 0, height: 0 })
const resizeHandle = ref<string>('')

// Constants
const HANDLE_SIZE = 8
const MIN_CROP_SIZE = 20

// Load image and initialize canvas
const loadImage = async (src: string) => {
  if (!src) return
  
  isLoading.value = true
  emit('ready', false)
  
  try {
    const img = new Image()
    
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve()
      img.onerror = (error) => {
        console.error('Image load error:', error)
        reject(error)
      }
      
      // Don't set crossOrigin to avoid canvas tainting issues
      // Convert local file paths to proper Tauri URLs
      if (src.startsWith('/') || src.match(/^[A-Za-z]:/)) {
        // This is a local file path
        img.src = convertFileSrc(src)
      } else {
        // This is a URL or data URL
        img.src = src
      }
    })
    
    originalImage.value = img
    
    // Use nextTick to ensure canvas is properly mounted before drawing
    await nextTick()
    
    // Small delay to ensure canvas is fully ready
    setTimeout(() => {
      initializeCanvas()
      initializeCropArea()
      drawCanvas()
      emit('ready', true)
    }, 10)
  } catch (error) {
    console.error('Failed to load image:', error)
    emit('ready', false)
  } finally {
    isLoading.value = false
  }
}

// Initialize canvas dimensions
const initializeCanvas = () => {
  if (!originalImage.value || !canvasRef.value) return
  
  const container = canvasRef.value.parentElement
  if (!container) return
  
  const containerRect = container.getBoundingClientRect()
  const maxWidth = Math.min(800, containerRect.width - 32) // Account for padding
  const maxHeight = Math.min(600, containerRect.height - 32)
  
  const img = originalImage.value
  const imgAspect = img.width / img.height
  const containerAspect = maxWidth / maxHeight
  
  if (imgAspect > containerAspect) {
    canvasWidth.value = maxWidth
    canvasHeight.value = maxWidth / imgAspect
  } else {
    canvasHeight.value = maxHeight
    canvasWidth.value = maxHeight * imgAspect
  }
  
  // Calculate image scale and position for centering
  const scaleX = canvasWidth.value / img.width
  const scaleY = canvasHeight.value / img.height
  imageScale.value = Math.min(scaleX, scaleY) * props.zoom
  
  imageOffsetX.value = (canvasWidth.value - img.width * imageScale.value) / 2
  imageOffsetY.value = (canvasHeight.value - img.height * imageScale.value) / 2
}

// Initialize crop area based on ratio
const initializeCropArea = () => {
  if (!originalImage.value) return
  
  const centerX = canvasWidth.value / 2
  const centerY = canvasHeight.value / 2
  
  let width: number
  let height: number
  
  switch (props.ratio) {
    case '1:1':
      width = height = Math.min(canvasWidth.value, canvasHeight.value) * 0.6
      break
    case '16:9':
      width = Math.min(canvasWidth.value * 0.8, canvasHeight.value * 0.8 * (16/9))
      height = width / (16/9)
      break
    case '4:3':
      width = Math.min(canvasWidth.value * 0.8, canvasHeight.value * 0.8 * (4/3))
      height = width / (4/3)
      break
    case 'free':
    default:
      width = canvasWidth.value * 0.6
      height = canvasHeight.value * 0.6
      break
  }
  
  cropArea.value = {
    x: centerX - width / 2,
    y: centerY - height / 2,
    width,
    height
  }
  
  constrainCropArea()
  emit('crop-change', cropArea.value)
}

// Constrain crop area to canvas bounds and maintain aspect ratio
const constrainCropArea = () => {
  const crop = cropArea.value
  
  // Maintain aspect ratio if needed
  if (props.ratio !== 'free') {
    let aspectRatio: number
    switch (props.ratio) {
      case '1:1':
        aspectRatio = 1
        break
      case '16:9':
        aspectRatio = 16/9
        break
      case '4:3':
        aspectRatio = 4/3
        break
      default:
        aspectRatio = crop.width / crop.height
    }
    
    if (isResizing.value) {
      // When resizing, adjust height based on width
      crop.height = crop.width / aspectRatio
    }
  }
  
  // Ensure minimum size
  crop.width = Math.max(MIN_CROP_SIZE, crop.width)
  crop.height = Math.max(MIN_CROP_SIZE, crop.height)
  
  // Constrain to canvas bounds
  crop.x = Math.max(0, Math.min(crop.x, canvasWidth.value - crop.width))
  crop.y = Math.max(0, Math.min(crop.y, canvasHeight.value - crop.height))
  crop.width = Math.min(crop.width, canvasWidth.value - crop.x)
  crop.height = Math.min(crop.height, canvasHeight.value - crop.y)
}

// Draw canvas with image and crop overlay
const drawCanvas = () => {
  const canvas = canvasRef.value
  if (!canvas || !originalImage.value) {
    console.log('Canvas or image not ready for drawing')
    return
  }
  
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    console.log('Canvas context not available')
    return
  }
  
  try {
    // Clear canvas
    ctx.clearRect(0, 0, canvasWidth.value, canvasHeight.value)
    
    // Draw image
    ctx.drawImage(
      originalImage.value,
      imageOffsetX.value,
      imageOffsetY.value,
      originalImage.value.width * imageScale.value,
      originalImage.value.height * imageScale.value
    )
    
    // Draw crop overlay
    drawCropOverlay(ctx)
  } catch (error) {
    console.error('Error drawing canvas:', error)
  }
}

// Draw crop overlay with handles and grid
const drawCropOverlay = (ctx: CanvasRenderingContext2D) => {
  const crop = cropArea.value
  
  // Draw dark overlay outside crop area
  ctx.fillStyle = 'rgba(0, 0, 0, 0.5)'
  
  // Top
  ctx.fillRect(0, 0, canvasWidth.value, crop.y)
  // Bottom
  ctx.fillRect(0, crop.y + crop.height, canvasWidth.value, canvasHeight.value - crop.y - crop.height)
  // Left
  ctx.fillRect(0, crop.y, crop.x, crop.height)
  // Right
  ctx.fillRect(crop.x + crop.width, crop.y, canvasWidth.value - crop.x - crop.width, crop.height)
  
  // Draw crop border
  ctx.strokeStyle = '#ffffff'
  ctx.lineWidth = 2
  ctx.strokeRect(crop.x, crop.y, crop.width, crop.height)
  
  // Draw grid lines (rule of thirds)
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)'
  ctx.lineWidth = 1
  
  // Vertical lines
  const thirdWidth = crop.width / 3
  ctx.beginPath()
  ctx.moveTo(crop.x + thirdWidth, crop.y)
  ctx.lineTo(crop.x + thirdWidth, crop.y + crop.height)
  ctx.moveTo(crop.x + thirdWidth * 2, crop.y)
  ctx.lineTo(crop.x + thirdWidth * 2, crop.y + crop.height)
  ctx.stroke()
  
  // Horizontal lines
  const thirdHeight = crop.height / 3
  ctx.beginPath()
  ctx.moveTo(crop.x, crop.y + thirdHeight)
  ctx.lineTo(crop.x + crop.width, crop.y + thirdHeight)
  ctx.moveTo(crop.x, crop.y + thirdHeight * 2)
  ctx.lineTo(crop.x + crop.width, crop.y + thirdHeight * 2)
  ctx.stroke()
  
  // Draw resize handles
  drawResizeHandles(ctx)
}

// Draw resize handles at corners and edges
const drawResizeHandles = (ctx: CanvasRenderingContext2D) => {
  const crop = cropArea.value
  const handleSize = HANDLE_SIZE
  const halfHandle = handleSize / 2
  
  ctx.fillStyle = '#ffffff'
  ctx.strokeStyle = '#000000'
  ctx.lineWidth = 1
  
  const handles = [
    // Corners
    { x: crop.x - halfHandle, y: crop.y - halfHandle, type: 'nw' },
    { x: crop.x + crop.width - halfHandle, y: crop.y - halfHandle, type: 'ne' },
    { x: crop.x - halfHandle, y: crop.y + crop.height - halfHandle, type: 'sw' },
    { x: crop.x + crop.width - halfHandle, y: crop.y + crop.height - halfHandle, type: 'se' },
  ]
  
  // Only add edge handles for free ratio
  if (props.ratio === 'free') {
    handles.push(
      { x: crop.x + crop.width / 2 - halfHandle, y: crop.y - halfHandle, type: 'n' },
      { x: crop.x + crop.width / 2 - halfHandle, y: crop.y + crop.height - halfHandle, type: 's' },
      { x: crop.x - halfHandle, y: crop.y + crop.height / 2 - halfHandle, type: 'w' },
      { x: crop.x + crop.width - halfHandle, y: crop.y + crop.height / 2 - halfHandle, type: 'e' }
    )
  }
  
  handles.forEach(handle => {
    ctx.fillRect(handle.x, handle.y, handleSize, handleSize)
    ctx.strokeRect(handle.x, handle.y, handleSize, handleSize)
  })
}

// Get resize handle at position
const getResizeHandle = (x: number, y: number): string => {
  const crop = cropArea.value
  const handleSize = HANDLE_SIZE
  const halfHandle = handleSize / 2
  
  const handles = [
    { x: crop.x - halfHandle, y: crop.y - halfHandle, type: 'nw' },
    { x: crop.x + crop.width - halfHandle, y: crop.y - halfHandle, type: 'ne' },
    { x: crop.x - halfHandle, y: crop.y + crop.height - halfHandle, type: 'sw' },
    { x: crop.x + crop.width - halfHandle, y: crop.y + crop.height - halfHandle, type: 'se' },
  ]
  
  if (props.ratio === 'free') {
    handles.push(
      { x: crop.x + crop.width / 2 - halfHandle, y: crop.y - halfHandle, type: 'n' },
      { x: crop.x + crop.width / 2 - halfHandle, y: crop.y + crop.height - halfHandle, type: 's' },
      { x: crop.x - halfHandle, y: crop.y + crop.height / 2 - halfHandle, type: 'w' },
      { x: crop.x + crop.width - halfHandle, y: crop.y + crop.height / 2 - halfHandle, type: 'e' }
    )
  }
  
  for (const handle of handles) {
    if (x >= handle.x && x <= handle.x + handleSize &&
        y >= handle.y && y <= handle.y + handleSize) {
      return handle.type
    }
  }
  
  return ''
}

// Check if point is inside crop area
const isInsideCropArea = (x: number, y: number): boolean => {
  const crop = cropArea.value
  return x >= crop.x && x <= crop.x + crop.width &&
         y >= crop.y && y <= crop.y + crop.height
}

// Mouse event handlers
const handleMouseDown = (event: MouseEvent) => {
  if (props.disabled) return
  
  const rect = canvasRef.value?.getBoundingClientRect()
  if (!rect) return
  
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  
  const handle = getResizeHandle(x, y)
  if (handle) {
    isResizing.value = true
    resizeHandle.value = handle
  } else if (isInsideCropArea(x, y)) {
    isDragging.value = true
  }
  
  dragStartX.value = x
  dragStartY.value = y
  dragStartCrop.value = { ...cropArea.value }
}

const handleMouseMove = (event: MouseEvent) => {
  if (props.disabled) return
  
  const rect = canvasRef.value?.getBoundingClientRect()
  if (!rect) return
  
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  
  // Update cursor
  if (!isDragging.value && !isResizing.value) {
    const handle = getResizeHandle(x, y)
    if (handle) {
      const cursors: Record<string, string> = {
        'nw': 'nw-resize',
        'ne': 'ne-resize',
        'sw': 'sw-resize',
        'se': 'se-resize',
        'n': 'n-resize',
        's': 's-resize',
        'w': 'w-resize',
        'e': 'e-resize'
      }
      canvasRef.value!.style.cursor = cursors[handle] || 'crosshair'
    } else if (isInsideCropArea(x, y)) {
      canvasRef.value!.style.cursor = 'move'
    } else {
      canvasRef.value!.style.cursor = 'crosshair'
    }
  }
  
  if (isDragging.value) {
    // Move crop area
    const deltaX = x - dragStartX.value
    const deltaY = y - dragStartY.value
    
    cropArea.value.x = dragStartCrop.value.x + deltaX
    cropArea.value.y = dragStartCrop.value.y + deltaY
    
    constrainCropArea()
    drawCanvas()
    emit('crop-change', cropArea.value)
  } else if (isResizing.value) {
    // Resize crop area
    const deltaX = x - dragStartX.value
    const deltaY = y - dragStartY.value
    
    const startCrop = dragStartCrop.value
    const handle = resizeHandle.value
    
    if (handle.includes('w')) {
      cropArea.value.x = startCrop.x + deltaX
      cropArea.value.width = startCrop.width - deltaX
    } else if (handle.includes('e')) {
      cropArea.value.width = startCrop.width + deltaX
    }
    
    if (handle.includes('n')) {
      cropArea.value.y = startCrop.y + deltaY
      cropArea.value.height = startCrop.height - deltaY
    } else if (handle.includes('s')) {
      cropArea.value.height = startCrop.height + deltaY
    }
    
    constrainCropArea()
    drawCanvas()
    emit('crop-change', cropArea.value)
  }
}

const handleMouseUp = () => {
  isDragging.value = false
  isResizing.value = false
  resizeHandle.value = ''
  if (canvasRef.value) {
    canvasRef.value.style.cursor = 'crosshair'
  }
}

// Touch event handlers
const handleTouchStart = (event: TouchEvent) => {
  if (event.touches.length === 1) {
    const touch = event.touches[0]
    const mouseEvent = new MouseEvent('mousedown', {
      clientX: touch.clientX,
      clientY: touch.clientY
    })
    handleMouseDown(mouseEvent)
  }
  event.preventDefault()
}

const handleTouchMove = (event: TouchEvent) => {
  if (event.touches.length === 1) {
    const touch = event.touches[0]
    const mouseEvent = new MouseEvent('mousemove', {
      clientX: touch.clientX,
      clientY: touch.clientY
    })
    handleMouseMove(mouseEvent)
  }
  event.preventDefault()
}

const handleTouchEnd = () => {
  handleMouseUp()
}

// Watch for prop changes
watch(() => props.imageSrc, (newSrc) => {
  if (newSrc) {
    loadImage(newSrc)
  }
}, { immediate: true })

watch(() => props.ratio, () => {
  initializeCropArea()
  drawCanvas()
})

watch(() => props.zoom, () => {
  initializeCanvas()
  drawCanvas()
})

// Convert current crop area from canvas coordinates to original image pixel coordinates.
// Use this when applying the crop so the correct region is used regardless of letterboxing/zoom.
const getCropInImageCoordinates = (): CropArea | null => {
  if (!originalImage.value || imageScale.value <= 0) return null
  const img = originalImage.value
  const crop = cropArea.value
  const x = (crop.x - imageOffsetX.value) / imageScale.value
  const y = (crop.y - imageOffsetY.value) / imageScale.value
  const w = crop.width / imageScale.value
  const h = crop.height / imageScale.value
  const x0 = Math.max(0, Math.min(x, img.width - 1))
  const y0 = Math.max(0, Math.min(y, img.height - 1))
  const x1 = Math.max(x0 + 1, Math.min(x + w, img.width))
  const y1 = Math.max(y0 + 1, Math.min(y + h, img.height))
  return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 }
}

// Expose crop area for parent component
defineExpose({
  getCropArea: () => cropArea.value,
  getCropInImageCoordinates,
  resetCrop: () => {
    initializeCropArea()
    drawCanvas()
  },
  forceRedraw: () => {
    drawCanvas()
  },
  canvasRef
})

onMounted(() => {
  if (props.imageSrc) {
    loadImage(props.imageSrc)
  }
})
</script>