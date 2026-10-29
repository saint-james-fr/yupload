use crate::domain::{FitMode, ImageProcessor, ImageStyle, ProcessOptions};
use crate::utils::download_image_safely;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};

// Quality mode toggle: true for balanced (faster), false for maximum quality (slower)
const USE_BALANCED_MODE: bool = true;

// Constants matching the TypeScript script
const BLUR_RADIUS: f32 = 12.0;
const BRIGHTNESS_VALUE: f32 = -0.2; // Maps to 0.8 multiplier
const CONTRAST_VALUE: f32 = -5.0;
const MAIN_IMAGE_RATIO: f32 = 1.0 / 2.3;

// Maximum dimension for input images before downsampling
const MAX_INPUT_DIMENSION: u32 = if USE_BALANCED_MODE {
    1600 // Balanced: faster processing
} else {
    2400 // Maximum: better quality
};

// Scale factor for background blur processing (lower = faster)
const BLUR_SCALE_FACTOR: f32 = if USE_BALANCED_MODE {
    0.4 // Balanced: 768x432 for 1080p output
} else {
    0.55 // Maximum: 1056x594 for 1080p output
};

pub struct ImageProcessorImpl;

impl ImageProcessor for ImageProcessorImpl {
    fn process_image_for_youtube(
        &self,
        input_path: &str,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()> {
        use std::time::Instant;
        let total_start = Instant::now();

        let load_start = Instant::now();

        // Use mozjpeg for JPEG files, fallback to standard loader
        let img = if input_path.to_lowercase().ends_with(".jpg")
            || input_path.to_lowercase().ends_with(".jpeg")
        {
            match self.load_jpeg_fast(input_path) {
                Ok(img) => img,
                Err(_) => image::open(input_path)?,
            }
        } else {
            image::open(input_path)?
        };

        tracing::debug!(
            "⏱️  Load: {:?} - Input: {}x{}",
            load_start.elapsed(),
            img.width(),
            img.height()
        );

        let process_start = Instant::now();
        let processed = self.create_illustube_style(&img, width, height)?;
        tracing::debug!("⏱️  Processing: {:?}", process_start.elapsed());

        // Save as JPEG (faster encoding and acceptable quality for video input)
        let save_start = Instant::now();
        processed
            .save_with_format(output_path, ImageFormat::Jpeg)
            .with_context(|| format!("Failed to save processed image: {}", output_path))?;
        tracing::debug!("⏱️  Save: {:?}", save_start.elapsed());

        tracing::debug!("✅ TOTAL: {:?}", total_start.elapsed());
        Ok(())
    }

    fn download_and_process_image(
        &self,
        url: &str,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()> {
        // Use safe download function with URL validation, size limits, and Content-Type checks
        let bytes = download_image_safely(url)
            .with_context(|| format!("Failed to safely download image from URL: {}", url))?;

        let img =
            image::load_from_memory(&bytes).with_context(|| "Failed to parse downloaded image")?;

        let processed = self.create_illustube_style(&img, width, height)?;

        processed
            .save_with_format(output_path, ImageFormat::Jpeg)
            .with_context(|| format!("Failed to save processed image: {}", output_path))?;

        Ok(())
    }

    fn process_for_video(
        &self,
        main_path: Option<&str>,
        main_url: Option<&str>,
        style: ImageStyle,
        options: &ProcessOptions,
        output_path: &str,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let processed = match style {
            ImageStyle::BackgroundOnly => {
                let bg_img = self.load_background_from_options(options)?;
                self.create_background_only(&bg_img, width, height)?
            }
            _ => {
                let img = self.load_image_from_path_or_url(main_path, main_url)?;
                match style {
                    ImageStyle::Automatic => self.create_illustube_style(&img, width, height)?,
                    ImageStyle::Fit16x9 => {
                        let fit = options.fit_mode.unwrap_or(FitMode::Contain);
                        self.create_fit_16x9(&img, width, height, fit)?
                    }
                    ImageStyle::CustomBackground => {
                        let bg_img = self.load_background_from_options(options)?;
                        self.create_custom_background(&img, &bg_img, width, height)?
                    }
                    ImageStyle::BackgroundOnly => unreachable!(),
                }
            }
        };
        processed
            .save_with_format(output_path, ImageFormat::Jpeg)
            .with_context(|| format!("Failed to save processed image: {}", output_path))?;
        Ok(())
    }
}

impl ImageProcessorImpl {
    /// Get image bytes from a data URL or http(s) URL. Data URLs are used when the frontend
    /// passes a cropped/edited image (e.g. after editing a Discogs image).
    fn bytes_from_data_or_http_url(url: &str) -> Result<Vec<u8>> {
        if url.starts_with("data:") {
            let base64_data = url
                .splitn(2, ',')
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid data URL format"))?;
            general_purpose::STANDARD
                .decode(base64_data.trim())
                .with_context(|| "Failed to decode data URL base64")
        } else {
            download_image_safely(url).with_context(|| format!("Failed to download: {}", url))
        }
    }

    /// Load image from local path or URL (http(s) or data URL).
    fn load_image_from_path_or_url(
        &self,
        path: Option<&str>,
        url: Option<&str>,
    ) -> Result<DynamicImage> {
        match (path, url) {
            (Some(p), _) => image::open(p).with_context(|| format!("Failed to load image: {}", p)),
            (_, Some(u)) => {
                let bytes = Self::bytes_from_data_or_http_url(u)?;
                image::load_from_memory(&bytes).with_context(|| "Failed to parse image")
            }
            _ => anyhow::bail!("Either main_path or main_url must be set"),
        }
    }

    /// Load background image from options (path or url; url can be http(s) or data URL).
    fn load_background_from_options(&self, options: &ProcessOptions) -> Result<DynamicImage> {
        match (
            options.background_path.as_deref(),
            options.background_url.as_deref(),
        ) {
            (Some(p), _) => {
                let bytes = std::fs::read(p)
                    .with_context(|| format!("Failed to read background: {}", p))?;
                image::load_from_memory(&bytes).with_context(|| "Failed to parse background image")
            }
            (_, Some(u)) => {
                let bytes = Self::bytes_from_data_or_http_url(u)
                    .with_context(|| format!("Failed to load background: {}", u))?;
                image::load_from_memory(&bytes).with_context(|| "Failed to parse background image")
            }
            _ => anyhow::bail!("CustomBackground requires background_path or background_url"),
        }
    }

    /// Fit 16:9: contain (letterbox) or cover (center crop).
    fn create_fit_16x9(
        &self,
        img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
        fit: FitMode,
    ) -> Result<DynamicImage> {
        let working = self.downsample_if_needed(img);
        let (w, h) = working.dimensions();
        let target_ratio = canvas_width as f32 / canvas_height as f32;
        let img_ratio = w as f32 / h as f32;

        match fit {
            FitMode::Contain => {
                let (tw, th) = if img_ratio > target_ratio {
                    (canvas_width, (canvas_width as f32 / img_ratio) as u32)
                } else {
                    ((canvas_height as f32 * img_ratio) as u32, canvas_height)
                };
                let scaled = self.fast_nearest_resize(&working, tw, th);
                let mut out = image::RgbaImage::new(canvas_width, canvas_height);
                let ox = (canvas_width.saturating_sub(tw)) / 2;
                let oy = (canvas_height.saturating_sub(th)) / 2;
                imageops::overlay(&mut out, &scaled, ox as i64, oy as i64);
                Ok(DynamicImage::ImageRgba8(out))
            }
            FitMode::Cover => {
                let scale = (canvas_width as f32 / w as f32).max(canvas_height as f32 / h as f32);
                let sw = (w as f32 * scale) as u32;
                let sh = (h as f32 * scale) as u32;
                let scaled = self.fast_nearest_resize(&working, sw, sh);
                let left = (sw.saturating_sub(canvas_width)) / 2;
                let top = (sh.saturating_sub(canvas_height)) / 2;
                let cropped =
                    imageops::crop_imm(&scaled, left, top, canvas_width, canvas_height).to_image();
                Ok(DynamicImage::ImageRgba8(cropped))
            }
        }
    }

    /// Custom background: bg covers canvas, main image centered at MAIN_IMAGE_RATIO.
    fn create_custom_background(
        &self,
        main_img: &DynamicImage,
        bg_img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<DynamicImage> {
        // Reuse the same background placement as background-only variant.
        let mut result = self.create_background_canvas(bg_img, canvas_width, canvas_height)?;

        // Then overlay the centered main image.
        let working_main = self.downsample_if_needed(main_img);
        let square_crop = self.crop_to_square(&working_main)?;
        let main_size = (canvas_width as f32 * MAIN_IMAGE_RATIO) as u32;
        let main_image = self.fast_nearest_resize(&square_crop, main_size, main_size);
        let mx = (canvas_width - main_size) / 2;
        let my = (canvas_height - main_size) / 2;
        imageops::overlay(&mut result, &main_image, mx as i64, my as i64);
        Ok(DynamicImage::ImageRgba8(result))
    }
    /// Background only: bg covers the canvas with no centered image.
    fn create_background_only(
        &self,
        bg_img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<DynamicImage> {
        let result = self.create_background_canvas(bg_img, canvas_width, canvas_height)?;
        Ok(DynamicImage::ImageRgba8(result))
    }

    /// Helper: scale/crop background to cover canvas.
    fn create_background_canvas(
        &self,
        bg_img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<image::RgbaImage> {
        let (bw, bh) = bg_img.dimensions();
        let scale = (canvas_width as f32 / bw as f32).max(canvas_height as f32 / bh as f32);
        let sw = (bw as f32 * scale) as u32;
        let sh = (bh as f32 * scale) as u32;
        let resized_bg = self.fast_nearest_resize(bg_img, sw, sh);
        let left = (sw.saturating_sub(canvas_width)) / 2;
        let top = (sh.saturating_sub(canvas_height)) / 2;
        let result =
            imageops::crop_imm(&resized_bg, left, top, canvas_width, canvas_height).to_image();
        Ok(result)
    }

    /// Loads JPEG files using mozjpeg decoder
    fn load_jpeg_fast(&self, path: &str) -> Result<DynamicImage> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Decompress with mozjpeg
        let decompress = mozjpeg::Decompress::new_mem(&buffer)?;
        let mut decompress = decompress.rgb()?;
        let pixels = decompress.read_scanlines()?;

        let width = decompress.width() as u32;
        let height = decompress.height() as u32;

        // Convert RGB to RGBA
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for chunk in pixels.chunks(3) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255); // Alpha
        }

        let img_buf = image::RgbaImage::from_raw(width, height, rgba)
            .ok_or_else(|| anyhow::anyhow!("Failed to create image from mozjpeg"))?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    }

    /// Resizes image using nearest-neighbor interpolation with parallel processing
    fn fast_nearest_resize(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> DynamicImage {
        use rayon::prelude::*;

        let (src_width, src_height) = img.dimensions();
        let x_ratio = src_width as f32 / target_width as f32;
        let y_ratio = src_height as f32 / target_height as f32;

        let rgba = img.to_rgba8();

        // Process rows in parallel
        let pixels: Vec<u8> = (0..target_height)
            .into_par_iter()
            .flat_map(|y| {
                let src_y = (y as f32 * y_ratio) as u32;
                let mut row = Vec::with_capacity((target_width * 4) as usize);

                for x in 0..target_width {
                    let src_x = (x as f32 * x_ratio) as u32;
                    let pixel = rgba.get_pixel(src_x, src_y);
                    row.extend_from_slice(&pixel.0);
                }
                row
            })
            .collect();

        let img_buf = image::RgbaImage::from_raw(target_width, target_height, pixels)
            .expect("Failed to create resized image");

        DynamicImage::ImageRgba8(img_buf)
    }

    /// Downsamples large images to fit within MAX_INPUT_DIMENSION
    fn downsample_if_needed(&self, img: &DynamicImage) -> DynamicImage {
        let (width, height) = img.dimensions();
        let max_dim = width.max(height);

        if max_dim <= MAX_INPUT_DIMENSION {
            return img.clone();
        }

        let scale = MAX_INPUT_DIMENSION as f32 / max_dim as f32;
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;

        tracing::debug!(
            "Downsampling from {}x{} to {}x{}",
            width,
            height,
            new_width,
            new_height
        );

        self.fast_nearest_resize(img, new_width, new_height)
    }

    /// Creates an Illustube-style image with blurred/darkened background and centered square crop
    fn create_illustube_style(
        &self,
        img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<DynamicImage> {
        use std::time::Instant;

        // Downsample large images to working size
        let ds_start = Instant::now();
        let working_img = self.downsample_if_needed(img);
        tracing::debug!("  ⏱️  Downsample: {:?}", ds_start.elapsed());

        // Create blurred/darkened background
        let bg_start = Instant::now();
        let background = self.create_background(&working_img, canvas_width, canvas_height)?;
        tracing::debug!("  ⏱️  Background: {:?}", bg_start.elapsed());

        // Crop to square
        let crop_start = Instant::now();
        let square_crop = self.crop_to_square(&working_img)?;
        tracing::debug!("  ⏱️  Crop: {:?}", crop_start.elapsed());

        // Calculate main image size and position
        let main_image_size = (canvas_width as f32 * MAIN_IMAGE_RATIO) as u32;
        let main_image_x = (canvas_width - main_image_size) / 2;
        let main_image_y = (canvas_height - main_image_size) / 2;

        // Resize square crop to target size
        let resize_start = Instant::now();
        let main_image = self.fast_nearest_resize(&square_crop, main_image_size, main_image_size);
        tracing::debug!("  ⏱️  Main resize: {:?}", resize_start.elapsed());

        // Composite main image onto background
        let comp_start = Instant::now();
        let mut result = background.to_rgba8();
        imageops::overlay(
            &mut result,
            &main_image,
            main_image_x as i64,
            main_image_y as i64,
        );
        tracing::debug!("  ⏱️  Composite: {:?}", comp_start.elapsed());

        Ok(DynamicImage::ImageRgba8(result))
    }

    /// Creates a blurred and darkened background that covers the canvas
    fn create_background(
        &self,
        img: &DynamicImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<DynamicImage> {
        use std::time::Instant;
        let (img_width, img_height) = img.dimensions();

        // Process background at reduced resolution
        let blur_width = (canvas_width as f32 * BLUR_SCALE_FACTOR) as u32;
        let blur_height = (canvas_height as f32 * BLUR_SCALE_FACTOR) as u32;
        tracing::debug!(
            "    Processing background at {}x{}",
            blur_width,
            blur_height
        );

        // Calculate scale to cover the canvas
        let scale_x = blur_width as f32 / img_width as f32;
        let scale_y = blur_height as f32 / img_height as f32;
        let scale = scale_x.max(scale_y);

        let scaled_width = (img_width as f32 * scale) as u32;
        let scaled_height = (img_height as f32 * scale) as u32;

        // Resize to cover blur canvas
        let resize1_start = Instant::now();
        let resized = self.fast_nearest_resize(img, scaled_width, scaled_height);
        tracing::debug!(
            "      Resize to {}x{}: {:?}",
            scaled_width,
            scaled_height,
            resize1_start.elapsed()
        );

        // Center crop to blur canvas size
        let left = (scaled_width.saturating_sub(blur_width)) / 2;
        let top = (scaled_height.saturating_sub(blur_height)) / 2;

        let crop_start = Instant::now();
        let cropped_buffer =
            imageops::crop_imm(&resized, left, top, blur_width, blur_height).to_image();
        let cropped = DynamicImage::ImageRgba8(cropped_buffer);
        tracing::debug!("      Crop: {:?}", crop_start.elapsed());

        // Apply blur (radius scaled to match resolution)
        let blur_radius = BLUR_RADIUS * BLUR_SCALE_FACTOR;
        let blur_start = Instant::now();
        let blurred = self.apply_blur(&cropped, blur_radius)?;
        tracing::debug!("      Blur (r={}): {:?}", blur_radius, blur_start.elapsed());

        // Apply brightness and contrast adjustments
        let bc_start = Instant::now();
        let adjusted =
            self.apply_brightness_contrast(&blurred, BRIGHTNESS_VALUE, CONTRAST_VALUE)?;
        tracing::debug!("      Brightness/Contrast: {:?}", bc_start.elapsed());

        // Upscale to final canvas size
        let resize2_start = Instant::now();
        let final_bg = self.fast_nearest_resize(&adjusted, canvas_width, canvas_height);
        tracing::debug!(
            "      Final resize to {}x{}: {:?}",
            canvas_width,
            canvas_height,
            resize2_start.elapsed()
        );

        Ok(final_bg)
    }

    /// Crops the image to a centered square
    fn crop_to_square(&self, img: &DynamicImage) -> Result<DynamicImage> {
        let (width, height) = img.dimensions();

        if width == height {
            return Ok(img.clone());
        }

        let size = width.min(height);
        let left = (width - size) / 2;
        let top = (height - size) / 2;

        let cropped_buffer = imageops::crop_imm(img, left, top, size, size).to_image();
        Ok(DynamicImage::ImageRgba8(cropped_buffer))
    }

    /// Applies Gaussian blur to the image
    fn apply_blur(&self, img: &DynamicImage, radius: f32) -> Result<DynamicImage> {
        if radius <= 0.0 {
            return Ok(img.clone());
        }

        Ok(img.blur(radius))
    }

    /// Applies brightness and contrast adjustments using parallel processing
    fn apply_brightness_contrast(
        &self,
        img: &DynamicImage,
        brightness: f32,
        contrast: f32,
    ) -> Result<DynamicImage> {
        use rayon::prelude::*;

        let brightness_mult = 1.0 + brightness;
        let contrast_mult = 1.0 + (contrast / 100.0);
        let contrast_offset = -(contrast / 2.0);
        let combined_mult = brightness_mult * contrast_mult;

        let mut result = img.to_rgba8();

        // Process rows in parallel
        result
            .enumerate_rows_mut()
            .par_bridge()
            .for_each(|(_y, row)| {
                for (_x, _y, pixel) in row {
                    pixel[0] = ((pixel[0] as f32 * combined_mult + contrast_offset)
                        .clamp(0.0, 255.0)) as u8;
                    pixel[1] = ((pixel[1] as f32 * combined_mult + contrast_offset)
                        .clamp(0.0, 255.0)) as u8;
                    pixel[2] = ((pixel[2] as f32 * combined_mult + contrast_offset)
                        .clamp(0.0, 255.0)) as u8;
                }
            });

        Ok(DynamicImage::ImageRgba8(result))
    }
}
