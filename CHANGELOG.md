# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Release notes are written in English and are intended to be copied directly
into the **"What's new"** field of Google Play Console when publishing a release.

## [Unreleased]

## [0.2.5] - 2026-08-07

### Changed

- Series filter dropdown rebuilt for mobile:
  - Responsive layout: brand chips in a horizontally scrollable strip (with fade-mask affordance) on small screens; the classic two-column layout is kept on desktop.
  - The first brand is auto-selected when the panel opens, so the series grid is never empty.
  - Panel width/height are clamped to the viewport, so it never overflows the screen edges on narrow devices.
  - Brand chips restyled to match Flowbite alternative buttons (dark mode text is now readable).

### Fixed

- Dropdown panel corner radius was covered by inner button hover backgrounds on desktop (panel overflow clipping restored).
- Fade mask on the chip strip was not visible on first open (layout timing); it now appears immediately and tracks scroll/resize.

## [0.2.4] - 2026-08-05

### Added

- Source Colors panel overhaul:
  - Drag to reorder cards with the grip handle.
  - Temporarily hide a source from the gamut with the eye toggle (card is kept, state persists).
  - Swipe left/right to delete cards on touch screens.
- A default "My Stock" card is created on first use only.
- Play channel: in-app update flow via Google Play In-app Updates (store builds prompt the official update flow).
- Sideload channel: update check now reads version.json from the GitHub Release asset — the request is routed through the Rust side (tauri-plugin-http), so WebView CORS no longer blocks it.

### Changed

- Android app size optimized: release profile (LTO + stripping) and arm64-only sideload builds (~46 MB → ~9 MB APK).
- Refined touch interactions (input vs. swipe conflicts resolved; hidden cards still draggable).
- Update checker refactored: channel-based factory + subclass design (Play / GitHub).
- CI: on tag push, the release AAB is automatically uploaded to Google Play Console (internal testing track).

### Fixed

- My Stock card no longer reappears after being deleted.
- Play channel update check: no longer reports an error when up to date, and correctly detects an available update.

## [0.2.3] - 2026-08-04

### Added

- Update checker on the About page (Tauri apps): detects new versions and links to your store or GitHub release page to review the changelog before updating.

### Changed

- Refined Chinese translations.

## [0.2.2] - 2026-08-04

### Added

- Bilingual interface: the app now supports **English** and **中文**, with a language switcher on the About page.
- Discord community link on the About page.

### Changed

- Android app icon and package name updated for the Play Store release.

### Fixed

- (none)

## [0.2.1] - 2026-07-?

### Added

- Camera color picker (Tauri only): point the rear camera at a surface to sample its color.

### Changed

- UI adapted for the Tauri (mobile) environment.

## [0.2.0] - 2026-07-?

### Added

- Paint catalog and personal stock management.
- Color search: find the closest paint to any HSL/RGB color, with optional multi-paint mixing.
- 3D OKLab gamut visualization.

## [0.1.0] - 2026-07-?

### Added

- Initial release: color picker, paint data loading via WebAssembly, project scaffolding.

[0.2.5]: https://github.com/cos-y/paintbox/releases/tag/v0.2.5
[0.2.4]: https://github.com/cos-y/paintbox/releases/tag/v0.2.4
[0.2.3]: https://github.com/cos-y/paintbox/releases/tag/v0.2.3
[0.2.2]: https://github.com/cos-y/paintbox/releases/tag/v0.2.2
[0.2.1]: https://github.com/cos-y/paintbox/releases/tag/v0.2.1
[0.2.0]: https://github.com/cos-y/paintbox/releases/tag/v0.2.0
[0.1.0]: https://github.com/cos-y/paintbox/releases/tag/v0.1.0
