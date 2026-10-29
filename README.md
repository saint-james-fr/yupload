# YouTube Rust Tool

A professional desktop application for creating YouTube videos from audio files and images, with integrated upload management and audio processing capabilities.

## Architecture

- **Frontend**: Vue 3 with Composition API, ShadCn UI, Tailwind CSS
- **Backend**: Rust with modular architecture (domain/use-cases/infrastructure pattern)
- **Video Processing**: FFmpeg (command-line)
- **Image Processing**: Rust `image` crate (Sharp equivalent)
- **Platform**: Tauri (cross-platform desktop app)

## Features

### 🎬 Video Creation
- **Bulk Video Creation**: Pair multiple audio tracks with images to create videos in parallel
- **Flexible Image Sources**: Use local image files, folder images, or image URLs
- **Smart Image Assignment**: Assign same image to all tracks or customize per track
- **Image Processing**: Automatic processing to 1920x1080 YouTube format with blurred backgrounds
- **Progress Tracking**: Real-time progress for each video with detailed status updates
- **Audio Normalization**: Optional audio normalization during video creation

### 📤 YouTube Upload
- **Bulk Upload**: Upload multiple videos to YouTube simultaneously
- **Rich Metadata**: Custom titles, descriptions, tags, categories, and privacy settings
- **Discogs Integration**: Auto-fill metadata from Discogs releases
  - Search and link Discogs releases to videos
  - Auto-populate descriptions, tags, and metadata
  - Link releases to multiple videos at once
- **Video Thumbnails**: Automatic thumbnail generation from videos
- **OAuth Authentication**: Secure YouTube API authentication
- **Upload Progress**: Track upload progress for each video individually

### 🎵 Audio Normalization
- **Standalone Normalization**: Normalize audio files independently of video creation
- **Loudness Normalization**: Uses loudnorm (EBU R128) with peak adjustment
- **Parallel Processing**: Process multiple files simultaneously using all CPU cores
- **USB-Compatible Output**: Optimized settings for USB playback (44.1kHz, 16-bit, stereo)
- **High-Pass Filtering**: Automatic filtering to remove low-frequency noise

### ⚙️ Settings & Account Management
- **YouTube Connection**: Connect/disconnect YouTube account
- **Discogs Connection**: Connect/disconnect Discogs account
- **OAuth Management**: Secure token storage and refresh

## Setup

1. **Configure OAuth Credentials**:
   Create a `.env` file in the project root (see `.env.example`):
   ```bash
   cp .env.example .env
   # Edit .env with your OAuth credentials
   ```
   
   **Required Environment Variables:**
   - `YOUTUBE_CLIENT_ID` - Google OAuth client ID
   - `YOUTUBE_CLIENT_SECRET` - Google OAuth client secret (OPTIONAL - see note below)
   - `DISCOGS_CONSUMER_KEY` - Discogs API consumer key
   - `DISCOGS_CONSUMER_SECRET` - Discogs API consumer secret
   
   **IMPORTANT - Google OAuth Client Type:**
   - When creating OAuth client in Google Cloud Console, use **"UWP" (Universal Windows Platform)** type
   - **DO NOT** use "Desktop app" type - it incorrectly requires `client_secret` even with PKCE (Google's quirk)
   - With UWP type, `YOUTUBE_CLIENT_SECRET` is **NOT required** - PKCE-only flow works correctly
   - This follows OAuth 2.0 best practices: desktop apps should not store secrets
   
   **Note on Security:**
   - With UWP client type, no `client_secret` is needed (proper PKCE implementation)
   - Credentials will be embedded in the compiled binary
   - Use environment variables at build time (not committed to source control)

2. Install dependencies:
   ```bash
   yarn install
   ```

2. Install FFmpeg (required for video creation):
   - macOS: `brew install ffmpeg`
   - Linux: `sudo apt-get install ffmpeg` (or equivalent)
   - Windows: Download from https://ffmpeg.org/

3. Run development server:
   ```bash
   yarn tauri dev
   ```

## Project Structure

```
src-tauri/
  src/
    domain/          # Domain models and traits
    use_cases/       # Business logic use cases
    infrastructure/ # Implementations (image processor, video creator, file handler)
    lib.rs          # Tauri commands and app setup
src/
  App.vue          # Main Vue component
  types.ts         # TypeScript type definitions
  lib/utils.ts     # Utility functions (cn for class merging)
  style.css        # Tailwind CSS setup
```

## Usage

### Video Creation
1. **Select Audio Files**: Choose a folder or select individual audio files
2. **Assign Images**: For each audio track, assign an image (file, folder image, or URL)
3. **Configure Options**: Choose output folder and enable audio normalization if needed
4. **Create Videos**: Generate multiple videos in parallel with real-time progress tracking

### YouTube Upload
1. **Authenticate**: Connect your YouTube account in Settings
2. **Select Videos**: Choose video files or a folder containing videos
3. **Edit Metadata**: Add titles, descriptions, tags, and privacy settings for each video
4. **Link Discogs Releases** (Optional): Search and link Discogs releases for automatic metadata
5. **Upload**: Bulk upload all videos with progress tracking

### Audio Normalization
1. **Select Audio Files**: Choose files or a folder containing audio files
2. **Select Output Folder**: Choose where normalized files will be saved
3. **Normalize**: Process files in parallel with progress tracking

## Technical Details

### Video Creation
- **Image Processing**: Images processed to 1920x1080 using Illustube-style algorithm:
  - Blurred/darkened background (blur: 12px, brightness: -0.2, contrast: -5)
  - Centered square crop overlay (1/2.3 of canvas width)
- **Video Encoding**: H.264 codec with AAC audio (YouTube-compatible)
- **Output Format**: MP4 with 30fps
- **Parallel Processing**: Multiple videos created simultaneously (up to 4 concurrent)
- **Image Preview**: Preview processed images before video creation

### Audio Normalization
- **Algorithm**: EBU R128 loudness normalization (loudnorm) with exact peak adjustment
- **Target Levels**: 
  - Integrated loudness: -16 LUFS
  - True peak: -0.5 dBTP
  - Final peak: -0.1 dB
- **High-Pass Filter**: 30Hz cutoff to remove low-frequency noise
- **Output Format**: WAV (44.1kHz, 16-bit, stereo) for USB compatibility

### YouTube Integration
- **API**: YouTube Data API v3
- **OAuth 2.0**: Desktop application flow with local callback server
- **Upload Method**: Resumable uploads with progress tracking
- **Metadata Support**: Full YouTube metadata including categories, privacy, and made-for-kids settings

### Discogs Integration
- **API**: Discogs API v2
- **OAuth 1.0a**: Three-legged OAuth flow
- **Features**: Release search, metadata extraction, description generation

## UI/UX Features

- **Modern Interface**: ShadCn UI components with Tailwind CSS
- **Real-Time Progress**: Detailed progress tracking for all operations
- **Two-Panel Layout**: Efficient workflow for bulk operations
- **Image Thumbnails**: Fast thumbnail generation for folder browsing
- **Video Thumbnails**: Automatic thumbnail extraction from videos
- **Status Alerts**: Clear success/error messaging
- **Responsive Design**: Works on different screen sizes

## FFmpeg Support

- **Bundled FFmpeg**: Support for bundled FFmpeg binaries (see FFMPEG_BUNDLING.md)
- **System FFmpeg**: Falls back to system FFmpeg if bundled version not available
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Product Analysis

For a comprehensive analysis of missing features and recommendations, see [PRODUCT_ANALYSIS.md](./PRODUCT_ANALYSIS.md).
