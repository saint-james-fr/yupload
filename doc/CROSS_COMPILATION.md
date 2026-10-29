# Cross-Compilation Guide

This guide explains how to build the application for different platforms from your macOS machine.

## Mac Intel (x86_64) - ✅ Supported

### Prerequisites

1. Install the Rust target:
```bash
rustup target add x86_64-apple-darwin
```

2. Install Xcode Command Line Tools (if not already installed):
```bash
xcode-select --install
```

### Building for Intel Mac

**Single architecture (Intel only):**
```bash
yarn tauri build --target x86_64-apple-darwin
```

**Universal binary (both Intel and Apple Silicon):**
```bash
yarn tauri build --target universal-apple-darwin
```

**Note**: Universal binaries are larger but work on both architectures. Tauri will automatically create a universal binary if you don't specify a target.

### FFmpeg for Intel Mac

If you're bundling FFmpeg, make sure to include the Intel version:
- Download from https://evermeet.cx/ffmpeg/
- Place in `src-tauri/binaries/macos/x86_64/ffmpeg`

## Windows - ✅ Cross-Compilation Supported (with limitations)

You **can** build Windows binaries from macOS using cross-compilation! However, there are some limitations:
- **NSIS installers**: Supported via cross-compilation
- **MSI installers**: Can only be created on Windows
- Some native dependencies may have issues

### Option 1: Cross-Compile from macOS (Experimental but Working)

**Prerequisites:**

1. Install mingw-w64 (GNU toolchain for Windows):
```bash
brew install mingw-w64
```

2. Add the Windows GNU target to Rust (NOT as default toolchain):
```bash
rustup target add x86_64-pc-windows-gnu
```

3. Configure Cargo to use mingw-w64 linker. Create or edit `~/.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"
```

**Note**: If the linker is not found, you may need to add the full path. On macOS with Homebrew, it's typically:
- Apple Silicon: `/opt/homebrew/bin/x86_64-w64-mingw32-gcc`
- Intel Mac: `/usr/local/bin/x86_64-w64-mingw32-gcc`

Or ensure mingw-w64 binaries are in your PATH:
```bash
export PATH="/opt/homebrew/opt/mingw-w64/bin:$PATH"
```

**Building for Windows:**

```bash
yarn tauri build --target x86_64-pc-windows-gnu
```

**Important Notes:**
- Do NOT set `x86_64-pc-windows-gnu` as your default Rust toolchain
- Use `rustup target add` to add it as a target, then specify with `--target` flag
- This will create NSIS installers (`.exe`), not MSI installers
- Some Rust crates with native dependencies may not cross-compile well

**Troubleshooting:**
- If you get linker errors, ensure mingw-w64 is properly installed and in PATH
- Some crates may need Windows-specific features disabled
- For MSI installers, you'll need to build on Windows or use CI/CD

### Option 2: GitHub Actions (Recommended for MSI)

Create `.github/workflows/build.yml`:

```yaml
name: Build

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: ''
          - platform: 'windows-latest'
            args: ''
          - platform: 'ubuntu-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
      
      - name: Install dependencies
        run: yarn install
      
      - name: Build
        run: yarn tauri build
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}
          path: src-tauri/target/release/bundle/**
```

### Option 3: Windows VM or Machine

If you have access to a Windows machine or VM:

1. Install prerequisites:
   - Node.js and Yarn
   - Rust (via rustup)
   - Visual Studio Build Tools (for C++ compilation)

2. Build:
```bash
yarn install
yarn tauri build
```

**Note**: Building on Windows allows you to create MSI installers, which cannot be cross-compiled from macOS.

### Option 4: Docker (Advanced)

You can use Docker with a Windows container, but this is complex and not recommended for Tauri apps due to GUI requirements.

## Current Build Configuration

Your `tauri.conf.json` is already configured for cross-platform builds:
- `"targets": "all"` in bundle config will build for all supported targets
- Resources are configured to include architecture-specific binaries

## Testing Cross-Platform Builds

### Mac Intel
- Use Rosetta 2 to test Intel builds on Apple Silicon:
```bash
arch -x86_64 /path/to/your/app
```

### Windows
- Test on a Windows VM or use GitHub Actions artifacts
- Consider using services like AppVeyor or Azure Pipelines

## Troubleshooting

### Mac Intel Build Issues

1. **Linker errors**: Ensure Xcode Command Line Tools are installed
2. **Missing dependencies**: Some native dependencies may need to be built for x86_64
3. **FFmpeg path**: Verify FFmpeg binary is in the correct architecture-specific directory

### Windows Build Issues

1. **Use CI/CD**: GitHub Actions is the most reliable option
2. **Native dependencies**: Some Rust crates may have Windows-specific requirements
3. **Code signing**: Windows builds require code signing for distribution

## Recommended Workflow

1. **Development**: Build locally for your current architecture
2. **Mac Intel**: Build locally using `--target x86_64-apple-darwin`
3. **Windows NSIS**: Cross-compile from macOS using `--target x86_64-pc-windows-gnu`
4. **Windows MSI**: Use GitHub Actions or Windows machine for MSI installers
5. **Release**: Tag releases to trigger multi-platform builds via CI/CD

## Installer Types

- **NSIS (.exe)**: Can be cross-compiled from macOS ✅
- **MSI (.msi)**: Requires Windows machine or CI/CD ⚠️
