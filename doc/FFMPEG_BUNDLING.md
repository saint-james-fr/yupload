# FFmpeg Bundling Configuration

This document describes how to bundle FFmpeg with the Tauri application.

## Option 1: Bundle FFmpeg Binaries (Recommended for Distribution)

### macOS
**Architecture Support**: Both Intel (x86_64) and Apple Silicon (ARM64) are supported.

1. Download FFmpeg static builds from https://evermeet.cx/ffmpeg/
   - For Intel Macs: Download `ffmpeg` (x86_64)
   - For Apple Silicon: Download `ffmpeg` (ARM64)
2. Place binaries in the appropriate subdirectories:
   - Intel: `src-tauri/binaries/macos/x86_64/ffmpeg`
   - ARM64: `src-tauri/binaries/macos/arm64/ffmpeg`
3. The application will automatically detect the architecture at runtime and use the correct binary.

**Note**: When building for a specific target (e.g., macOS ARM64), Tauri bundles ALL binaries matching the `binaries/**/*` pattern. This means:
- ✅ **macOS Universal Builds**: Both x86_64 and ARM64 binaries are bundled, allowing the app to run on both Intel and Apple Silicon Macs
- ⚠️ **Platform-Specific Builds**: If you build for macOS ARM64, binaries for Linux and Windows are also included (increasing bundle size unnecessarily)

**Recommendation**: For production builds targeting a specific platform, consider using a build script to copy only the relevant binaries to a staging directory before bundling. However, for macOS universal builds, bundling both architectures is intentional and beneficial.

### Linux
**Architecture Support**: Primary support for x86_64 (glibc), which covers most desktop Linux distributions (Ubuntu, Debian, Fedora, Arch, etc.).

1. Download FFmpeg static build:
   - **Recommended**: x86_64 glibc build (covers 95%+ of desktop Linux users)
   - Sources: https://johnvansickle.com/ffmpeg/ (static builds) or build from source
2. Place `ffmpeg` binary in `src-tauri/binaries/linux/x86_64/ffmpeg`
3. Optional: For ARM64 Linux support, place in `src-tauri/binaries/linux/arm64/ffmpeg`

**Distribution Coverage**:
- ✅ Ubuntu/Debian (x86_64)
- ✅ Fedora/RHEL (x86_64)
- ✅ Arch Linux (x86_64)
- ✅ openSUSE (x86_64)
- ⚠️ Alpine Linux (musl) - not currently supported, would require separate musl build

### Windows
**Architecture Support**: Single x86_64 build covers Windows 7 through Windows 11.

1. Download FFmpeg static build from https://www.gyan.dev/ffmpeg/builds/
   - Use the "release builds" - x86_64 version
   - Compatible with Windows 7, 8, 10, and 11
2. Extract and place `ffmpeg.exe` in `src-tauri/binaries/windows/x86_64/ffmpeg.exe`
3. Optional: For ARM64 Windows support, place in `src-tauri/binaries/windows/arm64/ffmpeg.exe`

## Option 2: Use System FFmpeg (Current Implementation)

The current implementation uses system FFmpeg. Users must have FFmpeg installed:
- macOS: `brew install ffmpeg`
- Linux: `sudo apt-get install ffmpeg` (or equivalent)
- Windows: Download from https://ffmpeg.org/

## Implementation Notes

The FFmpeg path detection in `src-tauri/src/utils/ffmpeg.rs` uses **Tauri's PathResolver API** (recommended approach):

1. **Primary Method**: `get_ffmpeg_path_from_app(app_handle: &AppHandle)` uses Tauri's `PathResolver::resolve()` with `BaseDirectory::Resource` to automatically resolve the correct resource path on all platforms.

2. **Fallback Method**: `get_ffmpeg_path()` uses manual path construction matching Tauri's documented resource locations:
   - **macOS**: `${exe_dir}/../Resources/binaries/macos/{arch}/ffmpeg` (inside .app bundle)
   - **Linux**: 
     - Development: `${exe_dir}/../lib/${exe_name}/binaries/linux/{arch}/ffmpeg`
     - AppImage: `${APPDIR}/usr/lib/${exe_name}/binaries/linux/{arch}/ffmpeg`
     - Production: `/usr/lib/${exe_name}/binaries/linux/{arch}/ffmpeg`
   - **Windows**: Directory containing executable (`binaries/windows/{arch}/ffmpeg.exe`)

3. **Architecture Detection**: Automatically detects runtime architecture (x86_64 vs ARM64) and selects the appropriate binary.

4. **Fallback**: Falls back to system FFmpeg if no bundled binary is found.

**Important**: The code follows Tauri v2's recommended approach by using `PathResolver::resolve()` when `AppHandle` is available. This ensures compatibility with Tauri's resource bundling system across all platforms.

## Directory Structure

```
src-tauri/binaries/
├── macos/
│   ├── x86_64/
│   │   └── ffmpeg
│   └── arm64/
│       └── ffmpeg
├── linux/
│   ├── x86_64/
│   │   └── ffmpeg
│   └── arm64/          # Optional
│       └── ffmpeg
└── windows/
    ├── x86_64/
    │   └── ffmpeg.exe
    └── arm64/          # Optional
        └── ffmpeg.exe
```

## Build Configuration

The `tauri.conf.json` should include binaries in the bundle:
```json
{
  "bundle": {
    "resources": [
      "binaries/**/*"
    ]
  }
}
```

## Architecture Detection

The application uses Rust's `std::env::consts::ARCH` to detect the runtime architecture:
- macOS: `x86_64` or `aarch64` (ARM64)
- Linux: `x86_64` or `aarch64` (ARM64)
- Windows: `x86_64` or `aarch64` (ARM64)

This ensures the correct binary is selected automatically without user intervention.
