# Predefined background templates

Place image files here. They are **auto-discovered** and listed in the “Or from library” strip.

**Supported formats:** `.jpg`, `.jpeg`, `.png`, `.webp`, `.gif`

**No code changes needed:** any image file in this directory is picked up at runtime. The label in the UI is derived from the filename (e.g. `anton-darius-jvR9ieZVG64-unsplash.webp` → “Anton Darius JvR9ieZVG64 Unsplash”).

**Optional overrides:** To pin specific ids/labels/filenames, add entries to `PREDEFINED_BACKGROUNDS` in `src-tauri/src/utils/predefined_backgrounds.rs`. If that list is non-empty, only those entries are shown; if empty, the app scans this directory.

Files are resolved from the app resource directory (`backgrounds/`).
