# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Release notes are written in English and are intended to be copied directly
into the **"What's new"** field of Google Play Console when publishing a release.

## [0.3.0] - 2026-08-11

### Added

- Three color sources on the search page: palette, camera (Tauri only) and **from paint**. Picking a paint anchors the color to it — the swatch shows the brand logo, an info card lists the paint details with a readonly hex value, and a paint search box swaps the anchored paint.
- Paint detail card: opens a mobile bottom sheet (or desktop overlay) with full paint info, add-to-stock toggle, mix-from-stock and view-all-similar actions.
- Paint search box with keyboard navigation (arrows/enter/escape), auto-highlighted first result.
- Stock page navigation survives leaving and returning to the page: brand → series → paint level plus the sort, search and filter configuration are kept in a runtime store instead of being reset by route switching.
- Visual color bar strip above the search results.

### Changed

- Sliders (color channels, ranges) rewritten around pointer-event dragging through a shared composable: exact 1:1 touch mapping with no dead zones, keyboard accessibility preserved.
- Desktop layout (≥640px): page content capped at 1440px and centered; the top area (swatch + picker) keeps its natural width with whitespace on the sides, while the search area (filter row + results) spans the full width; result cards capped at 220px.
- The color swatch is always visible on the search page in every color-source mode, rendered at a consistent position.

### Fixed

- Mobile slider dead zones: the visual thumb no longer diverges from the finger (Chrome's native range mapping over the 44px touch thumb caused up to 22px of error at the track edges).
- Bottom sheet dragging: whole-card drag with proper touch handling; catching a card mid-close animation no longer leaves it stuck halfway.
- The paint anchor on the search page was silently cleared when the page loaded in palette mode; it is now only cleared when the color is changed through a non-paint channel.

## [0.2.11] - 2026-08-10

### Added

- Lazy page loading: Search, Stock and Gamut pages now render through a shared lazy shell (`LazyPage`) that dynamically imports the page implementation only after the layout load (wasm init + data fetch) has finished. Page code can no longer run before the paint data is ready, which removes the `wasmReady` rendering gate and the need for per-module wasm-readiness checks.

### Changed

- Stock store simplified to a plain set of paint ids; paint indices are resolved on the fly from the paint data instead of being cached per entry.
- Stale stock entries (paints no longer present in the data source, e.g. after a catalog id update) are automatically pruned, persisted and skipped during iteration — fixing search failures where an invalid index reached the wasm search, the deserialization errored (`-1.0, expected usize`) and the page was left with zero results.

### Removed

- Direct-equivalence catalog (brand cross-reference, e.g. Gunze H9 ↔ Gunze C9) from the wasm backend and frontend; color-based lookalikes remain available through nearest-color search.

## [0.2.10] - 2026-08-10

### Added

- Full Chinese localization of all paint names (2,151 paints across Gunze, Tamiya, AK and Vallejo): translated names are shown in the zh locale and matched by the stock/gamut name search.
- English names normalized for Gunze paints (all-caps catalog names converted to proper title case, e.g. CERULEAN BLUE → Cerulean Blue), added to en.json together with Tamiya's English translations.
- Per-language paint dictionaries now cover every brand: zh.json and en.json each contain all 2,151 paints, so the app fetches only the dictionary matching the user's language.

### Changed

- Paint names are resolved through a new `i18ndyn` module that loads only the language-specific JSON up front and swaps dictionaries live when the language is switched on the About page (no reload).
- Paint data slimmed down: the `desc` column was removed from colors.csv and from the wasm search result (smaller data file); display names are now looked up from the translation dictionaries by brand + code.

### Fixed

- Switching from Search to Gamut no longer freezes the UI: the 3D scene (three.js / @threlte Canvas) is now loaded lazily after the page renders, with a loading placeholder shown while WebGL initializes (previously the main thread blocked for several seconds).

## [0.2.9] - 2026-08-10

### Changed

- Page-level runtime state hoisted into module stores so it survives page navigation:
  - Search results are cached as a pure function of (color, filter, stock) — returning to the search page shows the previous results without re-searching, and results update only when an input actually changes.
  - Gamut view state (clip ranges, camera position/target/zoom) and the WebAssembly gamut object persist across page switches instead of being torn down and rebuilt.
- Surface-type filtering rewritten as bitmask flags with compact integer serialization (still accepts the legacy string/array format), making the search filter check a single AND instead of a set lookup.
- Stock entries now carry the paint index (lazy-resolved once the paint data is loaded) and the stock store exposes an entries() iterator.
- Root layout gates page rendering on a wasmReady flag, so routes never mount before the wasm paint data is initialized.

## [0.2.7] - 2026-08-08

### Added

- Stock page: brand-wide cross-series search (fuzzy match on paint code/name); the series list filters to only matching series, and the right pane follows the selected series.
- Stock page: sort menu in the header (by hue, saturation, lightness, in-stock first), applied to both the normal view and search results.
- Stock page: mobile series sidebar redesigned as compact thumbnail cards (stock badge + paint count), matching the search page filter style.
- README badges added (release version, build status, repo stats, tech stack).
- Gunze 2026 catalog data: H301-H340 color values updated from the new catalog; GX series names translated to English.

### Changed

- Stock page breadcrumb now shows the current series; the header shows the brand name in a title row with the breadcrumb below in smaller text.
- Search box expand/collapse uses gentle fade transitions; the header height stays stable, and on desktop the title stays visible with the search box right-aligned at half width.

### Fixed

- Switching series while searching now updates the right pane to the selected series (previously it stayed on global results).
- Closing the search box no longer flickers the header/breadcrumb (the title area stays in layout flow and the input overlays it absolutely).

## [0.2.6] - 2026-08-07

### Added

- Gunze (Gunze Sangyo) H series paint data (H1-H417 and more) added to the paint library.
- Camera in landscape now covers the full right pane (no rounded corners) and locks page scrolling, so the viewfinder fills the screen and hides all other content.
- Camera follows device orientation: portrait requests a vertical stream (3:4 viewfinder), landscape requests a horizontal one; rotating the device re-acquires the stream in the new orientation.
- Camera capture button moved inside the viewfinder — bottom-center in portrait, right-center in landscape — styled as a classic round white shutter.
- Camera live color swatch and hex code shown at the top-left of the viewfinder.
- Camera viewfinder renders as a full-width 16:9 placeholder while launching, avoiding a layout jump before the stream is ready.
- Gamut cards now fade out (opacity) while being swiped away to delete.
- Tauri: pinch-zoom is disabled inside the app.
- Tauri: debug builds use "[debug] paintbox" as the Android launcher label and window title, so debug and release builds are easy to tell apart.

### Changed

- Color swatch is pinned to the top of the search page while scrolling (mobile).
- Similarity function tuned for better color matching results.
- Source switcher (palette/camera) is now an icon-only, connected button group, placed consistently above both modes.
- Python scripts and CSV data files moved under /scripts for a cleaner repository root.

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

[0.2.11]: https://github.com/cos-y/paintbox/releases/tag/v0.2.11
[0.2.10]: https://github.com/cos-y/paintbox/releases/tag/v0.2.10
[0.2.9]: https://github.com/cos-y/paintbox/releases/tag/v0.2.9
[0.2.7]: https://github.com/cos-y/paintbox/releases/tag/v0.2.7
[0.2.6]: https://github.com/cos-y/paintbox/releases/tag/v0.2.6
[0.2.5]: https://github.com/cos-y/paintbox/releases/tag/v0.2.5
[0.2.4]: https://github.com/cos-y/paintbox/releases/tag/v0.2.4
[0.2.3]: https://github.com/cos-y/paintbox/releases/tag/v0.2.3
[0.2.2]: https://github.com/cos-y/paintbox/releases/tag/v0.2.2
[0.2.1]: https://github.com/cos-y/paintbox/releases/tag/v0.2.1
[0.2.0]: https://github.com/cos-y/paintbox/releases/tag/v0.2.0
[0.1.0]: https://github.com/cos-y/paintbox/releases/tag/v0.1.0
