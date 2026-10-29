# Changelog

All notable changes to Yupload will be documented in this file.

## [0.2.0] - 2026-02-02

### Added
- **Unified upload flow**: Image preview URL tracking and thumbnail handling for better user feedback during create & upload
- **Thumbnail support**: Thumbnail URL tracking for video items and thumbnail support in upload flow (PipelineProgress, VideoUpload)
- **Bulk actions**: Apply category and tags to all videos; apply privacy settings to multiple videos in upload process
- **File selection**: SourceStepWithFileCard component for improved file selection in upload process
- **Image handling**: Paste support and loading state in ImageAssignmentPanel; applyImageFromUrl in ImageAssignmentPanel and UnifiedUploadPage
- **Image sources**: Support for loading images from data URLs in ImageProcessor
- **Image editing**: Cropping and state management in ImageAssignmentPanel (ImageEditingModal, ImageCropTool)
- **Path display**: PathDisplay component integrated across components for path visualization and management
- **Audio export**: Save normalized audio files to a specified directory during video normalization process
- **FAQ**: Entry on link truncation and verification requirements for clickable links in YouTube descriptions
- **Concurrent uploads**: Concurrent video upload processing with metadata tracking and error handling
- **Cancellation**: Item cancellation in PipelineProgress and UnifiedUploadPage during uploads
- **Documentation**: Error handling and logging standards for Rust backend
- **Video metadata**: Video metadata types and VideoUploadMetadata structure for upload flow
- **YouTube connection**: Warning message and improved access control for upload actions in MainMenu
- **Upload flow**: Reset state and improved audio track removal handling in useUnifiedUploadFlow

### Fixed
- **Labels**: Unified upload action label "Upload to YouTube" → "Create & Upload" in SuccessPage
- **Image cropping**: Improved handling of image sources and coordinates in ImageEditingModal and ImageCropTool
- **Temp files**: cleanup_old_temp_files now processes only files (prevents errors with directories)
- **Preview**: Clearer preview rendering message in ImageAssignmentPanel
- **Discogs search**: Scroll-to-top for search results in DiscogsSearchResults
- **FAQ**: Clarified that audio normalization can be done during upload
- **Upload request**: Upload request structure includes request object for bulk video uploads
- **Messages**: Removed exclamation mark from success message in creation and upload flows
- **Pipeline progress**: Phase description includes waiting state for video creation; item messages alongside step labels; step labels reflect completion status

### Improved
- **YouTube options**: Privacy options and categories from computed properties; output folder selection via reusable components
- **Architecture**: Documentation updated with component choices rationale and structural guidelines
- **Upload flow**: Removed UnifiedUploadSourceStep and VideoUpload component; streamlined create & upload flow
- **Codebase**: Removed unused components (ExportActions, NormalizeAudioOption, Progress); cleaned VideoUpload auth sections
- **Callout**: "Action" variant added and used in VideoUpload, UnifiedUploadPage, ImageAssignmentPanel
- **FileSelectionCard**: Consistent icon sizes and padding across file selection interfaces
- **BottomNavigation**: ChevronRight icon and adjusted button styling
- **Path management**: Removed success feedback for path management
- **Commands**: Command usage replaced with create_hidden_command for audio normalization, logging, and client interactions
- **ImageAssignmentPanel**: Improved image and background file selection UI (accessibility and visual feedback)

### Technical
- Default logging level changed from "info" to "debug"
- Enhanced error handling and logging across Discogs and YouTube client commands

## [0.1.2] - 2026-01-30

### Added
- **Code Quality**: Added comprehensive verification rules with TypeScript and Rust type checking (`yarn verify`)
- **CI/CD**: Added GitHub Actions workflow for automated building and releasing across platforms
- **Deployment**: Added deployment script and package configuration for streamlined releases
- **Documentation**: Added macOS security warning and resolution steps to README
- **UI Enhancement**: Added Callout component to VideoMetadataEditor for better user guidance on description templates
- **FFmpeg Reliability**: Enhanced FFmpeg path resolution and functionality checks with comprehensive codec testing

### Fixed
- **Type Safety**: Standardized image style naming from "fit-16x9" to "fit16x9" across all components and preferences
- **Spelling**: Standardized "Yupload" spelling across all documentation and scripts
- **Logging**: Improved structured tracing and enhanced log export functionality
- **System Detection**: Enhanced OS version detection for macOS, Windows, and Linux

### Improved
- **Code Quality**: Added verification protocol requiring TypeScript and Rust checks before completion
- **Development Workflow**: Enhanced package.json scripts with `check:types`, `check:rust`, and `verify` commands
- **Documentation**: Cleaner project structure with improved .gitignore and documentation organization
- **User Experience**: Better description input guidance with contextual hints for template creation
- **Build Process**: Enhanced GitHub Actions with proper permissions and credential management

### Technical
- Migrated to consistent "fit16x9" naming convention across TypeScript types and Vue components
- Enhanced FFmpeg functionality validation with comprehensive codec availability checks
- Improved logging infrastructure with better structured tracing and export capabilities
- Added comprehensive verification protocol to prevent regressions and ensure code quality
- Enhanced deployment process with automated GitHub Actions workflow for multi-platform builds

## [0.1.1] - 2026-01-29

### Fixed
- **Critical**: Fixed image style serialization error ("unknown variant fit-16x9") by adding missing "background-only" validation
- **Critical**: Fixed WebView2 dependency issue on Windows by adding embedBootstrapper bundle configuration
- **Critical**: Fixed package.json dependency versions to use exact available versions
- **Critical**: Fixed logging system initialization order - OAuth server logs now properly captured
- **Critical**: Fixed logging guard lifecycle - prevents logging channels from shutting down prematurely
- **Critical**: Fixed log export functionality - added missing notify function import
- **Critical**: Fixed syntax errors in logging infrastructure

### Added
- **Logging System**: Comprehensive application logging with file rotation (20MB limit, 5 backup files)
- **Log Export**: User-selectable file dialog for exporting logs (replaces automatic Downloads folder)
- **System Information**: Automatic logging of OS version, memory, CPU, and dependency information on startup
- **FFmpeg Validation**: Silent startup check for FFmpeg availability, version, and codec support
- **Sensitive Data Protection**: Automatic redaction of tokens and secrets in logs
- **Bottom Navigation**: Added "Back to menu" buttons at bottom of all long pages
- **Scheduling Help**: Added descriptive text explaining scheduling is available for private videos
- **Panic Logging**: Application crashes are now logged before termination
- **Hint for Description Templates**: Added descriptive text explaining description templates are available for private videos

### Improved
- **Error Visibility**: Errors now persist in logs instead of disappearing quickly
- **Navigation**: Better user experience with consistent bottom navigation across pages
- **Upload Scheduling**: Clearer UI text and improved placeholder for scheduling input
- **Windows Compatibility**: Enhanced Windows deployment with proper WebView2 bundling
- **Logging Consistency**: Migrated all backend logging to structured tracing system
- **OS Version Detection**: Enhanced system information with detailed OS version reporting

### Technical
- Migrated from log macros to tracing-based logging infrastructure with structured output
- Added comprehensive logging to all critical operations (authentication, video creation, file operations)
- Enhanced video creation and bulk operations with detailed progress logging
- Updated all Tauri dependencies to compatible stable versions (CLI 2.9.6, API 2.9.1)
- Improved error handling and logging throughout the Rust backend
- Added system detection for FFmpeg functionality and WebView2 availability
- Implemented proper logging guard management to prevent channel shutdown
- Added application lifecycle logging (startup, shutdown, cleanup, panics)

## [0.1.0] - 2026-01-XX

### Added
- Initial release of Yupload
- Video creation from audio files and images
- YouTube upload functionality with bulk operations
- Audio normalization with EBU R128 standard
- Discogs integration for metadata auto-fill
- Multiple image processing styles (automatic, fit-16x9, custom-background, background-only)
- Upload history tracking and management
- Comprehensive settings and preferences system
- Cross-platform support (macOS, Windows, Linux)