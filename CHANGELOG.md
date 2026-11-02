# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Pre-release.** Until `1.0.0`, minor-version bumps may include breaking
> changes. Pin to a specific version or commit if you depend on this crate.

## [0.2.2] - 2026-11-02

### Added

#### Elements

- `input(...)` — new HTML `<input type="text">`-like element. Controlled via
  `.value(String)` / `.on_change(handler)`. Supports `.placeholder(...)`,
  `.mode(InputMode::Text | Password(char))`, `.max_length(n)`,
  `.auto_focus(bool)`, `.with_disabled(bool)`, `.with_read_only(bool)`, and
  `.on_submit(handler)` for Enter.
    - Password mode masks the display and the accessibility label — the raw
      password never flows through `a11y_alt`, and `AccessibilityRole` becomes
      `PasswordInput`.
    - Text selection: click-drag, shift+arrow, Ctrl+A. Password mode opts out (mask length may differ from value length
      in UTF-16).
    - Clipboard: Ctrl+C / Ctrl+V / Ctrl+X with `max_length` respect.
    - IME preedit rendered inline with an underline, positioned correctly at
      the cursor even mid-string.
    - Horizontal scrolling via `ScrollView` with a `follow_cursor` side effect
      so the cursor is always visible.
- `checkbox()` — new HTML `<input type="checkbox">`-like element. Controlled
  via `.with_checked(bool)`; activation via `.on_checked(...)` /
  `.on_unchecked(...)`. Custom icon via `.icon(..)`, custom icon color via
  `.check_color(..)`, box size via `.size(f32)`. `:checked` variant swaps
  `gray_100` → `blue_600` fill and shows a white lucide `check`.
- `radio()` — new HTML `<input type="radio">`-like element. Controlled via
  `.with_checked(bool)`; activation via `.on_checked(...)` only (HTML
  semantics: clicking a checked radio is a no-op). Custom dot color via
  `.dot_color(..)`, size via `.size(f32)`.
- `p(...)` — new HTML `<p>`-like paragraph wrapper. Ordered `.span(..)` /
  `.child(..)` content, `.max_lines(n)`, `.line_height(h)`, and HTML `<p>`
  UA defaults (`1em` top/bottom margin, `16px` at default font size).

#### Styling

- `Style::border: Option<Border>` — borders are now a first-class style
  property. Renders on `div`, `button`, `checkbox`, `radio`, `input`, and
  `image` (via `DecorationExt`); `text` and `p` ignore it (their primitives
  don't implement `DecorationExt`). Not inherited — matches CSS.
- Border utilities on `StyledExt`: `.border(..)`, `.border_none()`,
  `.border_0/1/2/4/8()`, `.border_t/r/b/l(..)`, `.border_x/y(..)`,
  `.border_width(..)`, `.border_color(..)`, `.border_inner/outer/center()`,
  `.border_alignment(..)`.
- `Style::merge_style(other)` — merge `other` into `self` with `other` taking
  precedence for fields that differ from `Style::bare()`'s default.
- `Style::inheritable_subset()` — the CSS-`inherit`-able slice of a `Style`:
  `text_color`, `text_align`, `font_*`, `text_*`, `letter_spacing`,
  `line_height`, `text_shadows`, `text_decoration`, `cursor`,
  `cursor_visible`. Everything else stays local to the element that set it.
- `InheritedStyle` newtype — context payload carrying the inherited slice.
- `styling::apply` — shared `Style` → primitive helpers: `apply_to_rect`,
  `apply_text_style`, `HasFontLists`, `apply_font_lists`.
- `styling::semantic_events` — shared semantic-event layer:
    - `SemanticEvents` — the eight universal pointer/focus callbacks.
    - `HasSemanticEvents` — fluent `.on_hover(..)` / `.on_pressed(..)` /
      `.on_focus(..)` / etc. setters, implemented on every interactive element.
    - `attach_pointer_handlers` — wires the four pointer events that drive
      `:hover` / `:active` and fire the semantic callbacks.
    - `use_semantic_focus_events` — custom hook that fires `.on_focus(..)` /
      `.on_blur(..)` / `.on_focus_visible(..)` exactly once per transition.
- `Div::with_semantic_events(..)` — replace the div's semantic-events bundle
  wholesale, useful when wrapping a `Div` inside another element.
- `Div::with_focusable(bool)` — force the focusability flag, overriding the
  default derivation. Removes the phantom `:focus` variant hack that every
  focusable element previously carried.
- `Div::scroll_y()` / `.scroll_x()` / `.with_scroll_controller(..)` /
  `.show_scrollbar(bool)` / `.hide_scrollbar()` — wrap children in a
  `ScrollView` while keeping the outer `rect()` (and all its styling)
  intact. Non-scrolling `div()`s behave identically to before.

#### Documentation

- Extensive module docs on `lib.rs`, `styling/mod.rs`, and `elements/mod.rs`:
  style-cascade diagram, variant precedence, disabled gate, semantic-event
  reference, border support matrix, scrolling reference, Tailwind-to-FreyaCN
  translation cheatsheet, FAQ, non-goals, and per-element known limitations.
- 5-step recipe for writing new elements, wired to `apply_to_rect` /
  `attach_pointer_handlers` / `use_semantic_focus_events`.
- README rewritten to reflect the current API surface, marked as
  pre-release, and updated to target Freya `0.5.0-rc.13`.

### Changed

- **Element constructors default to `Style::bare()` instead of
  `Style::default()`.** The theme is applied once, at the top of the tree,
  then inherited down through `InheritedStyle` context. Element-level
  `.style(..)` overrides now merge on top of the inherited slice — not on
  top of a theme-seeded `Style`.
- `Style::merge_style` uses a "differs from `Style::bare()`" test — a field
  whose value equals its bare default cannot be reset by a later merge. Every
  element with UA defaults exposes `.no_default_style()` as the escape hatch.
- `button()` — rewritten on top of `Div` with full semantic-event support.
  New `.on_selected(..)` / `.on_deselected(..)` activation callbacks. Focus
  is requested on click when focusable, matching `<button>`.
- `div()`, `text()`, `image()` — now consume `InheritedStyle` from the
  nearest ancestor and provide their own resolved style to descendants.
- `Style::default()` is no longer the recommended starting point for element
  styles. Use `Style::bare()` (the new element default) or let the element
  layer the theme itself.

### Fixed

- `input()` password mode uses `AccessibilityRole::PasswordInput` and masks
  the `a11y_alt` value so the real password never flows through the
  accessibility tree. Previously it declared `TextInput` and passed the raw
  value through `a11y_alt` — a real a11y leak.
- `checkbox()` / `radio()` now expose their checked state to assistive
  technology via `node.set_description(..)` (`"checked"` / `"unchecked"`,
  `"selected"` / `"not selected"`). This is a workaround for
  `accesskit::Toggled` not being reachable in the current dependency tree —
  the description supplements the label rather than replacing it, so custom
  labels set via `.a11y_alt(..)` survive.
- `div()` focusable elements now request focus on click when
  `.with_focusable(true)` or a `:focus`-family variant is present — matching
  `<div tabindex="0">`.

### Breaking

- `Button::selected(bool)` renamed to `Button::with_selected(bool)` so
  `StyledExt::selected(..)` remains callable for styling the `:selected`
  variant. Without the rename, `.selected(|s| ..)` resolved to the inherent
  setter and failed to typecheck.
- `Checkbox::checked(bool)` renamed to `Checkbox::with_checked(bool)`.
  `Radio::checked(bool)` renamed to `Radio::with_checked(bool)`. Same reason.
- `freya::icons::lucide::{image, radio, timer, *}` in the prelude replaced
  with `pub use freya::icons::lucide;` (module only, not its contents). The
  glob brought 1400+ icons into scope, and `image` / `radio` now conflict
  with the element constructors. Call `lucide::image()`, `lucide::radio()`,
  `lucide::timer()` explicitly.
- `Style::default()` is no longer the recommended base for element styles.
  Existing code that read `.style.text_color` / `.style.font_size` from an
  element before render will now see bare defaults (black, default size)
  instead of theme-resolved values.

## [0.2.1] - 2026-11-01

See the [`v0.2.1`](https://github.com/brightmadueke/freyacn/releases/tag/v0.2.1)
tag on GitHub for the change set.

[unreleased]: https://github.com/brightmadueke/freyacn/compare/v0.2.1...HEAD
[0.2.2]: https://github.com/brightmadueke/freyacn/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/brightmadueke/freyacn/releases/tag/v0.2.1