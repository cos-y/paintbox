# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Release notes are written in English and are intended to be copied directly
into the **"What's new"** field of Google Play Console when publishing a release.

## [0.4.2] - 2026-08-21

### Added

- Fuzzy search engine (Fuse.js) with tokenized AND matching ("gaia 042"), fuzzy subsequence matching ("prl" → Prism series), field-weighted scoring (code > serie > brand > tags > names), and debounced input.
- Finish/medium tags in multiple languages derived from paint surface bits: searching "珠光", "pearl" or "パール" finds pearl paints, "金属"/"metallic" finds metallic ones, etc.
- Cross-language name search: source-language color names (en/es/ja) load lazily on search-widget mount and merge into the index, so an English name like "brown" matches even in a Chinese UI.
- Browse-style search on the stock page uses the same engine while keeping series grouping and filter linkage.

### Changed

- Gunze PA (pearl) colors now prefer ground-truth data over interpolated values.
- Gaia paint data calibration tweaks.

### Fixed

- Detail-page comparison swatch now renders both halves in one coordinate space, so the finish pattern (light band / glow) flows continuously across the two cards.

## [0.4.1] - 2026-08-21

### Added

- Finish-aware paint rendering (pearl / metallic / clear / fluorescent) now covers the whole app, not just the stock gallery: the detail page's main swatch and comparison swatch (halves aligned so the finish pattern flows continuously across both), equivalent-paint rows, search-result cards, and the paint-search dropdowns.
- A shared mini-swatch component keeps list rows and dropdown thumbnails visually consistent.

### Changed

- Finish patterns now scale uniformly across swatch sizes, from 16px list thumbnails to full detail cards, keeping the light band, glow and sparkle geometry consistent.
- Gaia paint data calibration tweaks.

## [0.4.0] - 2026-08-21

### Added

- Pearl-finish swatch cards are now rendered from real color data with multi-base support; the Gaia pearl paints apply this calibration.
- The Gamut page gains a scatter mode and reverse paint-code lookup.

## [0.3.14] - 2026-08-18

### Added

- Semantic color comparison on search results: the raw ΔE and similarity percentage are replaced by a subjective similarity grade (Match / Very close / Close / Different, colored green→red) plus direction tags computed in Oklab — hue-axis offsets (redder / yellower / greener / bluer, with the blue-yellow axis weighted since it is perceptually more sensitive) and chroma direction (more vivid / grayer), localized in all four languages.

### Fixed

- Corrected element stacking on stock paint cards (removed an over-aggressive z-index from the stock badge and label overlay).

## [0.3.13] - 2026-08-18

### Added

- Citadel brand added (404 paints), sourced from the Miniature Painter Pro community database plus Hobby Color Converter cross-references, with full en/zh/ja/es translations and letter-coded series thumbnails (A–T).
- Illustrative rendering for special-finish swatches: metallic paints get a light band, pearlescent paints a dual-hue highlight (tinted away from the base color in both directions) with subtle sparkle, clear paints render as translucent color over a metallic grid, and fluorescent paints get a soft rectangular glow — a visual hint for the finish, without pretending to be photorealistic.

### Changed

- Citadel series thumbnails now use their letter series codes (A–T) instead of numeric suffixes.

## [0.3.12] - 2026-08-17

### Added

- 392 new Vallejo paints cross-referenced from the Miniature Painter Pro community database (github.com/Arcturus5404/miniature-paints), covering 7 series not previously collected: Arte Deco, Panzer Aces, Nocturna, Weathering FX, Surface Primer, Game Air (72.7xx numbering) and additional Game Color Xpress/Special FX/Wash entries, with full zh/ja/es translations.
- 33 new Gunze Mr.Hobby paints from the same source: 21 Aqueous Hobby Color weather-effect colors (H341–H466), 9 Mr Metal Color legacy metallics as a new MC series (MC211–MC219), 2 Mr Color Spray (S129/S151) and 1 Super Metallic (SM08), all with full translations.
- Gaia brand (167 paints) added.

### Changed

- Mediums reclassified to reflect design intent (not physical capability): Vallejo Game Air / Model Air / Mecha Color changed from airbrush+brush to airbrush-only (383 rows), Game Color changed to brush-only — official wording like "can also be applied with a brush" is treated as a compliance statement, not a brush flag.
- Surfaces corrected against official catalogs: Game Air and Game Color main line changed from semi-gloss to matt (143 rows), matching the "self-leveling matt finish" / "offer a matt finish" wording in the official PDFs.
- AV data adjusted against the sales script (950 rows touched) — serie/series labels and code prefixes reconciled with the sales tooling.
- AK equivalence link fixed (bad URL replaced).
- Wide-table merge dry-run no longer blocks on partial data: it warns and shows the diff, only rejecting on --apply.

### Fixed

- Equivalence symmetric-deduplication bug in build_data.py — equiv pairs are now deduplicated symmetrically, removing duplicate reverse links.

## [0.3.11] - 2026-08-16

### Added

- 574 new Vallejo paints from 11 official catalog PDFs (cc063, cc064, cc069, cc070, cc081, cc095, cc266, cc298, cc329, cc339, cc340), growing the Vallejo catalog from 4 to 12 series: Model Air, Primers, Mecha Color, Hobby Paint, Metal Color, Model Wash, Premium Color and True Metallic Metal join the existing Model Color, Liquid Metal, Game Color and Game Air.

### Changed

- Wide table moved to CSV storage (data/wide.csv) so bulk column edits stay possible with a text editor's column mode.
- Timestamps removed from data files — git itself is the change history, keeping diffs clean and line-oriented.

### Fixed

- Corrected series labels in the series picker: existing series names now include their color-code prefix (e.g. "Model Air [71.x]"), and Game Air had the wrong description (it previously repeated Game Color's).
- Surface-type bit order unified to the front-end convention (FL/PA bits swapped in the wide table), so fluorescent and pearlescent paints now show the correct surface label.

## [0.3.10] - 2026-08-15

### Added

- Official catalog traceability: paint detail cards show a small link badge at the top-right of the color swatch that opens the official catalog PDF the color was sourced from (a single source opens directly; multiple sources appear as a row of badges). Equivalence chips carry the same link.

### Changed

- Data pipeline reworked: color data now lives in a single wide table (data/wide.json) generated by a new Python toolchain (python/), and ships to the app as compact msgpack binaries (paints.bin / equivs.bin) loaded at runtime — smaller downloads and faster startup.
- Bundle size reduced to ~5 MB: WebAssembly release build re-tuned (opt-level 3, panic=abort, wasm-opt -O4) and three.js imports in the gamut scene switched to named imports for tree-shaking.
- Settings page wording adjusted.

### Fixed

- Paint name translation dictionaries were not being minified by the static-JSON plugin; the plugin was rewritten so the compressed output is actually applied.

## [0.3.9] - 2026-08-14

### Added

- Data backup & restore (Settings → Data): export your full paint library and preferences (stock, display-language preference, theme, interface language) to a JSON file — saved natively via a file dialog in the desktop app, downloaded as a file on web.
- Data import with validation: pick a backup file (native file dialog on desktop, file picker on web), the format is validated before anything is applied, and you choose the import scope — stock only, stock & settings, or settings only — plus, when stock is included, whether to append (deduplicated merge) or replace the entire stock (danger).
- The settings page now shows the Android-style numeric version code next to the semantic version (e.g. Version 0.3.9 (3009)), aligned with the Android build's versionCode.

### Changed

- Settings page layout: data backup & restore moved into its own section below the settings column.

## [0.3.8] - 2026-08-14

### Added

- Spanish localization: the full UI (navigation, settings, search, stock, gamut) is translated into Spanish, and the paint catalog gained a Spanish name dictionary covering all 2,291 paints — Vallejo keeps its official Spanish names (e.g. Cam. Marrón Medio) while AK Interactive, Gunze and Tamiya names are translated with proper Spanish word order (Dark Green → Verde Oscuro) and title-case formatting. The language is auto-detected for Spanish locales and selectable in Settings.
- Paint code, name, series and brand text on the paint-detail card (stock page) and the source-paint card (search page) can now be selected and copied in the Tauri desktop app, where text selection is otherwise disabled by default.

### Changed

- Spanish UI labels trimmed to well-known abbreviations where they were overly long: sort conditions (Sat., Lum.) and finish tags (SG, Transp., FL, Envej.).

## [0.3.7] - 2026-08-14

### Added

- Japanese localization: the full UI (navigation, settings, search, stock, gamut) is translated into Japanese, and the paint catalog gained a Japanese name dictionary covering all 2,291 paints — Gunze and Tamiya names keep their original Japanese forms (e.g. ホワイト（白）), while AK Interactive and Vallejo names are translated. The language is auto-detected for Japanese locales and selectable in Settings.
- Dedicated Settings page (route /settings) replacing the old About page: language picker, "prefer metadata in original language" toggle, and a theme picker with three modes (follow system / light / dark, default follow system) applied instantly without flash-on-load.
- Light theme: the entire UI was audited for light-mode readability — secondary helper text, tag pills, control borders and card edges all gained proper light-mode styling.

### Changed

- About page reworked into a proper Settings page; privacy, mixing-disclaimer and data-contribution sections are now collapsible.
- Control borders unified into a single theme-aware class that adapts to light/dark, and card shadows standardized across the stock, search and gamut pages.

### Fixed

- Low-contrast text in light mode across search, stock, gamut, settings and paint-detail views.
- Tag pill text was unreadable in light mode (dark background + dark text).

## [0.3.6] - 2026-08-14

### Added

- Gunze Gundam Marker catalog (series M): 123 marker paints covering the coating range (GM04-19), the EX series (XGM01-08/100/301/302 and hologram XGM201-205), panel line pens (GM01-03, GM20/21, GM301-313), Real Touch markers (GM400-410) and the 30MM / Plano-saurus ranges (TM, PSM), each with a swatch color sampled from the official catalog plus English/Chinese names.
- Medium filter: every paint is now tagged by application medium (airbrush, spray, brush, marker); the search and stock filters can restrict results to specific media.
- Tag-style filter controls: multi-select and single-select filters on the search and stock pages render as compact tag pills, and the series picker opens in the global bottom drawer on mobile.

### Changed

- Base type labels renamed to Lacquer / Alcohol / Enamel / Water, with an airbrush icon for the spray source.

## [0.3.5] - 2026-08-12

### Fixed

- localStorage data is now read through zod schemas with per-element recovery: corrupt or legacy entries are dropped individually instead of crashing the whole app (a legacy stock format could previously crash the module and lock the UI).

## [0.3.2] - 2026-08-12

### Added

- Desktop (≥640px) two-column layout on the search and stock pages: the paint list stays on the left while a persistent detail panel on the right shows the selected paint, or a hint when nothing is selected. The panel width adapts to the window.
- CIEDE2000 color distance replaces the previous Euclidean (Oklab) distance for color matching, for more perceptually accurate results.
- JetBrains Mono is bundled with the app and used as the default font on all platforms (Android and desktop).
- PanText: a single-line control for overlong text with a two-sided fading mask and drag-to-pan on mouse/pen.

### Changed

- Back navigation is now native-aware: on Android the system back button is handled by a JS listener registered on demand (drawer open, or a page level to pop), and falls back to the system default exit at the root level.
- Stock page navigation no longer uses the browser history API; brand/series/paint levels live in a runtime store.
- Media queries are reactive (isMedia()) so the layout adapts live on window resize.
- The bottom drawer content is now scrollable on touch; drag-to-close only engages when scrolled to the top.

### Fixed

- Desktop search results area overflowed the viewport; the results list and the detail panel now scroll independently within the remaining page height.
- The detail panel was shown on the stock brand list (root) level; it now only appears when a paint is selected.
- Stock page header buttons (search/filter/sort) disappeared at the paint level.
- Paint detail back required two presses to return to the brand list.

## [0.3.1] - 2026-08-11

- Bug fixes

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

[0.3.11]: https://github.com/cos-y/paintbox/releases/tag/v0.3.11
[0.3.12]: https://github.com/cos-y/paintbox/releases/tag/v0.3.12
[0.3.10]: https://github.com/cos-y/paintbox/releases/tag/v0.3.10
[0.3.9]: https://github.com/cos-y/paintbox/releases/tag/v0.3.9
[0.3.8]: https://github.com/cos-y/paintbox/releases/tag/v0.3.8
[0.3.7]: https://github.com/cos-y/paintbox/releases/tag/v0.3.7
[0.3.6]: https://github.com/cos-y/paintbox/releases/tag/v0.3.6
[0.3.1]: https://github.com/cos-y/paintbox/releases/tag/v0.3.1
[0.3.0]: https://github.com/cos-y/paintbox/releases/tag/v0.3.0
[0.2.11]: https://github.com/cos-y/paintbox/releases/tag/v0.2.11
[0.2.10]: https://github.com/cos-y/paintbox/releases/tag/v0.2.10
[0.2.9]: https://github.com/cos-y/paintbox/releases/tag/v0.2.9
[0.2.7]: https://github.com/cos-y/paintbox/releases/tag/v0.2.7
[0.3.14]: https://github.com/cos-y/paintbox/releases/tag/v0.3.14
[0.3.13]: https://github.com/cos-y/paintbox/releases/tag/v0.3.13
[0.2.6]: https://github.com/cos-y/paintbox/releases/tag/v0.2.6
[0.2.5]: https://github.com/cos-y/paintbox/releases/tag/v0.2.5
[0.2.4]: https://github.com/cos-y/paintbox/releases/tag/v0.2.4
[0.2.3]: https://github.com/cos-y/paintbox/releases/tag/v0.2.3
[0.2.2]: https://github.com/cos-y/paintbox/releases/tag/v0.2.2
[0.2.1]: https://github.com/cos-y/paintbox/releases/tag/v0.2.1
[0.2.0]: https://github.com/cos-y/paintbox/releases/tag/v0.2.0
[0.1.0]: https://github.com/cos-y/paintbox/releases/tag/v0.1.0
[0.4.0]: https://github.com/cos-y/paintbox/releases/tag/v0.4.0
[0.4.1]: https://github.com/cos-y/paintbox/releases/tag/v0.4.1
[0.4.2]: https://github.com/cos-y/paintbox/releases/tag/v0.4.2
