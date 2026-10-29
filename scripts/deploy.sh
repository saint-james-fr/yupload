#!/bin/bash

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Yupload Deployment Package Builder${NC}"
echo "========================================"

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Extract version from tauri.conf.json
echo -e "${YELLOW}📋 Extracting version from tauri.conf.json...${NC}"
VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*"version": "\([^"]*\)".*/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Error: Could not extract version from tauri.conf.json${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found version: $VERSION${NC}"

# Create deployment directory structure
DEPLOY_DIR="Yupload_$VERSION"
DEPLOY_BASE="$PROJECT_ROOT/deploy"
DEPLOY_PATH="$DEPLOY_BASE/$DEPLOY_DIR"

# Ensure deploy directory exists
mkdir -p "$DEPLOY_BASE"

echo -e "${YELLOW}📁 Creating deployment directory: $DEPLOY_DIR${NC}"

# Clean up existing deployment directory if it exists
if [ -d "$DEPLOY_PATH" ]; then
    echo -e "${YELLOW}🧹 Removing existing deployment directory...${NC}"
    rm -rf "$DEPLOY_PATH"
fi

# Create directory structure
mkdir -p "$DEPLOY_PATH"
mkdir -p "$DEPLOY_PATH/Windows"
mkdir -p "$DEPLOY_PATH/MacOS"

# Copy common files to root of deployment directory
echo -e "${YELLOW}📄 Copying asset files to root...${NC}"
if [ -d "deploy/assets" ]; then
    cp -r deploy/assets/* "$DEPLOY_PATH/" 2>/dev/null || echo -e "${YELLOW}⚠️  No files in deploy/assets directory${NC}"
else
    echo -e "${YELLOW}⚠️  deploy/assets directory not found, skipping...${NC}"
fi

# Copy CHANGELOG.md to root
echo -e "${YELLOW}📋 Copying CHANGELOG.md...${NC}"
if [ -f "CHANGELOG.md" ]; then
    cp "CHANGELOG.md" "$DEPLOY_PATH/"
    echo -e "${GREEN}✅ CHANGELOG.md copied${NC}"
else
    echo -e "${RED}❌ Warning: CHANGELOG.md not found${NC}"
fi

# Find and copy Windows build (.exe)
echo -e "${YELLOW}🪟 Looking for Windows build...${NC}"
# First try to find bundled installer (.exe in bundle directory)
WINDOWS_EXE=$(find src-tauri/target -path "*/bundle/*" -name "*.exe" -type f | grep -v "ffmpeg" | head -1)

# If no bundled installer found, look for the raw executable
if [ -z "$WINDOWS_EXE" ] || [ ! -f "$WINDOWS_EXE" ]; then
    echo -e "${YELLOW}   No bundled installer found, looking for raw executable...${NC}"
    WINDOWS_EXE=$(find src-tauri/target -path "*/release/yupload.exe" -type f | head -1)
fi

if [ -n "$WINDOWS_EXE" ] && [ -f "$WINDOWS_EXE" ]; then
    cp "$WINDOWS_EXE" "$DEPLOY_PATH/Windows/"
    echo -e "${GREEN}✅ Windows build copied: $(basename "$WINDOWS_EXE")${NC}"
    
    # Check if it's a raw executable (not an installer)
    if [[ "$WINDOWS_EXE" == *"/release/yupload.exe" ]]; then
        echo -e "${YELLOW}   Note: Raw executable copied (installer creation failed during cross-compilation)${NC}"
        echo -e "${YELLOW}   For a proper installer, build on Windows or use a Windows CI/CD${NC}"
    fi
else
    echo -e "${RED}❌ Warning: Windows .exe build not found in src-tauri/target${NC}"
    echo -e "${YELLOW}   Make sure to run: yarn tauri:build:windows${NC}"
    echo -e "${YELLOW}   Note: Cross-compilation from macOS may have limitations${NC}"
fi

# Find and copy macOS build (.dmg)
echo -e "${YELLOW}🍎 Looking for macOS build...${NC}"
MACOS_DMG=$(find src-tauri/target -name "*.dmg" -type f | grep -E "(release|bundle)" | head -1)

if [ -n "$MACOS_DMG" ] && [ -f "$MACOS_DMG" ]; then
    cp "$MACOS_DMG" "$DEPLOY_PATH/MacOS/"
    echo -e "${GREEN}✅ macOS build copied: $(basename "$MACOS_DMG")${NC}"
else
    echo -e "${RED}❌ Warning: macOS .dmg build not found in src-tauri/target${NC}"
    echo -e "${YELLOW}   Make sure to run: yarn tauri:build:macos${NC}"
fi

# Create ZIP archive
echo -e "${YELLOW}📦 Creating ZIP archive...${NC}"
ZIP_NAME="$DEPLOY_DIR.zip"
ZIP_PATH="$DEPLOY_BASE/$ZIP_NAME"

# Remove existing zip if it exists
if [ -f "$ZIP_PATH" ]; then
    rm "$ZIP_PATH"
fi

# Create zip archive (change to deploy directory to avoid full paths in zip)
if command -v zip >/dev/null 2>&1; then
    cd "$DEPLOY_BASE"
    zip -r "$ZIP_NAME" "$DEPLOY_DIR"
    cd "$PROJECT_ROOT"
    echo -e "${GREEN}✅ ZIP archive created: deploy/$ZIP_NAME${NC}"
    
    # Clean up the directory after successful ZIP creation
    echo -e "${YELLOW}🧹 Cleaning up deployment directory...${NC}"
    rm -rf "$DEPLOY_PATH"
    echo -e "${GREEN}✅ Deployment directory removed${NC}"
else
    echo -e "${RED}❌ Error: zip command not found${NC}"
    echo -e "${YELLOW}   Please install zip or manually compress the $DEPLOY_DIR directory${NC}"
    echo -e "${YELLOW}   Directory preserved: $DEPLOY_PATH${NC}"
fi

# Show directory structure before cleanup (if directory still exists)
if [ -d "$DEPLOY_PATH" ]; then
    echo ""
    echo -e "${BLUE}Directory structure:${NC}"
    tree "$DEPLOY_PATH" 2>/dev/null || find "$DEPLOY_PATH" -type f -exec echo "  {}" \;
fi

# Display final summary
echo ""
echo -e "${BLUE}📊 Deployment Summary${NC}"
echo "======================"
echo -e "${GREEN}Version:${NC} $VERSION"
echo -e "${GREEN}Package:${NC} $DEPLOY_DIR"
if [ -f "$ZIP_PATH" ]; then
    echo -e "${GREEN}Archive:${NC} deploy/$ZIP_NAME"
    echo -e "${GREEN}Size:${NC} $(du -h "$ZIP_PATH" | cut -f1)"
fi

echo ""
echo -e "${GREEN}🎉 Deployment package ready!${NC}"
if [ -f "$ZIP_PATH" ]; then
    echo -e "${YELLOW}📦 Archive: deploy/$ZIP_NAME${NC}"
    if [ ! -d "$DEPLOY_PATH" ]; then
        echo -e "${GREEN}✨ Clean deployment - only archive remains${NC}"
    fi
else
    echo -e "${YELLOW}📁 Directory: $DEPLOY_PATH${NC}"
fi