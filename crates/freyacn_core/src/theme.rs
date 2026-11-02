//! JSON-driven theming for FreyaCN, modeled on gpui-kit semantics.
//!
//! Uses [`StyledColor`] and the palette functions exported by
//! [`crate::styling::colors`]. Supports:
//!
//! - Solid colors, palette references (`{blue.500}`, `{neutral.950}/24`).
//! - Linear / radial / conic gradients.
//! - Free-form component tokens (`button.primary.background`, …).
//! - Component-state tokens (`hover`, `active`, `focus`, `disabled`, …)
//!   with fallback to the base token.

#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use freya::prelude::*;
use freya::prelude::{
    Readable, State, provide_context, try_consume_context, use_consume, use_hook,
};
use paste::paste;
use serde::{Deserialize, Serialize};

use crate::styling::colors::StyledColor;
use std::cell::RefCell;

thread_local! {
    /// The theme currently active on this thread. Read by
    /// [`Style::default`] to seed colors. Written by
    /// [`set_active_theme`] from `ThemeHandle`.
    static ACTIVE_THEME: RefCell<Option<Arc<Theme>>> = RefCell::new(None);
}

/// Install `theme` as the current thread's active theme.
///
/// Called automatically by [`load_theme_registry`] and
/// [`ThemeHandle::set_active`]. `Style::default()` calls created **after**
/// this point will pick up the theme's `background` / `foreground`
/// tokens.
pub fn set_active_theme(theme: Arc<Theme>) {
    ACTIVE_THEME.with(|slot| *slot.borrow_mut() = Some(theme));
}

/// Read the current thread's active theme.
///
/// Falls back to [`Theme::empty_light`] when no theme has been installed
/// (unit tests, headless runs, etc.), so `Style::default()` never panics.
pub fn active_theme() -> Arc<Theme> {
    ACTIVE_THEME
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| Arc::new(Theme::empty_light()))
}

// ---------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------

#[derive(Debug)]
pub enum ThemeError {
    Json(serde_json::Error),
    Io(std::io::Error),
    InvalidHex(String),
    InvalidPaletteRef(String),
    UnknownPaletteStep(String),
    UnknownTheme(String),
    NotASolidColor(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "json error: {e}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::InvalidHex(s) => write!(f, "invalid hex color: {s}"),
            Self::InvalidPaletteRef(s) => write!(f, "invalid palette reference: {s}"),
            Self::UnknownPaletteStep(s) => write!(f, "unknown palette step: {s}"),
            Self::UnknownTheme(s) => write!(f, "unknown theme id: {s}"),
            Self::NotASolidColor(s) => write!(f, "token is not a solid color: {s}"),
        }
    }
}
impl std::error::Error for ThemeError {}
impl From<serde_json::Error> for ThemeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
impl From<std::io::Error> for ThemeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ---------------------------------------------------------------------
// Appearance + interaction state
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}

/// Interaction / value states a component can enter. Mirrors gpui-kit's
/// set. `ComponentState::Base` is the fallback used when nothing is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentState {
    Base,
    Hover,
    Active,
    Focus,
    FocusVisible,
    Disabled,
    Checked,
    Indeterminate,
    Selected,
    Pressed,
    Open,
}

impl ComponentState {
    pub fn as_str(self) -> &'static str {
        match self {
            ComponentState::Base => "base",
            ComponentState::Hover => "hover",
            ComponentState::Active => "active",
            ComponentState::Focus => "focus",
            ComponentState::FocusVisible => "focus-visible",
            ComponentState::Disabled => "disabled",
            ComponentState::Checked => "checked",
            ComponentState::Indeterminate => "indeterminate",
            ComponentState::Selected => "selected",
            ComponentState::Pressed => "pressed",
            ComponentState::Open => "open",
        }
    }
}

// ---------------------------------------------------------------------
// JSON shapes
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    /// `"#fff"`, `"{blue.500}"`, `"{neutral.950}/24"`, `"transparent"`.
    Color(String),
    Linear {
        linear: LinearSpec,
    },
    Radial {
        radial: RadialSpec,
    },
    Conic {
        conic: ConicSpec,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stop {
    pub color: String,
    pub at: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearSpec {
    #[serde(default)]
    pub angle: f32,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialSpec {
    #[serde(default = "c05")]
    pub center: (f32, f32),
    #[serde(default = "one")]
    pub radius: f32,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConicSpec {
    #[serde(default)]
    pub angle: f32,
    #[serde(default = "c05")]
    pub center: (f32, f32),
    pub stops: Vec<Stop>,
}

fn c05() -> (f32, f32) {
    (0.5, 0.5)
}
fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypographyConfig {
    #[serde(default)]
    pub font_family: HashMap<String, String>,
    #[serde(default)]
    pub font_size: HashMap<String, (f32, f32)>,
    #[serde(default)]
    pub font_weight: HashMap<String, u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionConfig {
    #[serde(default)]
    pub duration: HashMap<String, u32>,
    #[serde(default)]
    pub easing: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub id: String,
    pub name: String,
    pub appearance: Appearance,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,

    /// Optional palette overrides on top of the built-in Tailwind scale.
    /// Key = `"group.step"`, value = hex string.
    #[serde(default)]
    pub palette: HashMap<String, String>,

    pub semantic: HashMap<String, TokenValue>,

    #[serde(default)]
    pub components: serde_json::Value,

    #[serde(default)]
    pub radius: HashMap<String, f32>,
    #[serde(default)]
    pub spacing: HashMap<String, f32>,
    #[serde(default)]
    pub typography: TypographyConfig,
    #[serde(default)]
    pub shadows: HashMap<String, String>,
    #[serde(default)]
    pub motion: MotionConfig,
    #[serde(default, rename = "zIndex")]
    pub z_index: HashMap<String, i32>,
}

// ---------------------------------------------------------------------
// Resolved theme
// ---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub appearance: Appearance,
    pub version: String,
    pub author: Option<String>,

    palette_overrides: HashMap<String, Color>,
    semantic: HashMap<String, Fill>,

    pub components: serde_json::Value,

    pub radius: HashMap<String, f32>,
    pub spacing: HashMap<String, f32>,
    pub typography: TypographyConfig,
    pub shadows: HashMap<String, String>,
    pub motion: MotionConfig,
    pub z_index: HashMap<String, i32>,
}

impl Theme {
    pub fn from_json_str(s: &str) -> Result<Self, ThemeError> {
        let cfg: ThemeConfig = serde_json::from_str(s)?;
        Self::from_config(cfg)
    }

    pub fn from_config(cfg: ThemeConfig) -> Result<Self, ThemeError> {
        let mut palette_overrides = HashMap::new();
        for (path, hex) in &cfg.palette {
            palette_overrides.insert(path.clone(), parse_hex(hex)?);
        }

        let empty_semantic = HashMap::new();
        let mut semantic = HashMap::new();
        for (key, val) in &cfg.semantic {
            // Semantic self-references are not resolved during this pass
            // (would be circular). Use plain palette refs in `semantic`;
            // use `{semantic.*}` inside `components`.
            let fill = resolve_fill(val, &palette_overrides, &empty_semantic)?;
            semantic.insert(key.clone(), fill);
        }

        Ok(Self {
            id: cfg.id,
            name: cfg.name,
            appearance: cfg.appearance,
            version: cfg.version,
            author: cfg.author,
            palette_overrides,
            semantic,
            components: cfg.components,
            radius: cfg.radius,
            spacing: cfg.spacing,
            typography: cfg.typography,
            shadows: cfg.shadows,
            motion: cfg.motion,
            z_index: cfg.z_index,
        })
    }

    // ---- Semantic lookups ----

    pub fn fill(&self, key: &str) -> Option<Fill> {
        self.semantic.get(key).cloned()
    }

    pub fn color(&self, key: &str) -> Option<Color> {
        match self.semantic.get(key)? {
            Fill::Color(c) => Some(*c),
            _ => None,
        }
    }

    // ---- Radius / spacing shortcuts ----

    pub fn radius(&self, key: &str) -> f32 {
        self.radius.get(key).copied().unwrap_or(8.0)
    }
    pub fn spacing(&self, key: &str) -> f32 {
        self.spacing.get(key).copied().unwrap_or(0.0)
    }

    // ---- Component token access ----

    pub fn component(&self, path: &str) -> Option<&serde_json::Value> {
        let mut cur = &self.components;
        for part in path.split('.') {
            cur = cur.get(part)?;
        }
        Some(cur)
    }

    pub fn component_color(&self, path: &str) -> Option<Color> {
        let v = self.component(path)?;
        let s = v.as_str()?;
        resolve_color(s, &self.palette_overrides, &self.semantic).ok()
    }

    pub fn component_fill(&self, path: &str) -> Option<Fill> {
        let v = self.component(path)?;
        if let Some(s) = v.as_str() {
            return resolve_color(s, &self.palette_overrides, &self.semantic)
                .ok()
                .map(Fill::Color);
        }
        serde_json::from_value::<TokenValue>(v.clone())
            .ok()
            .and_then(|t| resolve_fill(&t, &self.palette_overrides, &self.semantic).ok())
    }

    pub fn component_radius(&self, path: &str) -> Option<f32> {
        let v = self.component(path)?;
        if let Some(n) = v.as_f64() {
            return Some(n as f32);
        }
        if let Some(s) = v.as_str() {
            return self.radius.get(s).copied();
        }
        None
    }

    // ---- State-aware accessors ----

    /// Look up `{path}.{state}.{field}`, falling back to
    /// `{path}.{field}`. Precedence (e.g. `active` > `hover`) is the
    /// caller's responsibility — pass exactly one state.
    pub fn state_fill(&self, path: &str, state: ComponentState, field: &str) -> Option<Fill> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(f) = self.component_fill(&with_state) {
                return Some(f);
            }
        }
        self.component_fill(&format!("{}.{}", path, field))
    }

    pub fn state_color(&self, path: &str, state: ComponentState, field: &str) -> Option<Color> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(c) = self.component_color(&with_state) {
                return Some(c);
            }
        }
        self.component_color(&format!("{}.{}", path, field))
    }

    pub fn state_radius(&self, path: &str, state: ComponentState, field: &str) -> Option<f32> {
        if state != ComponentState::Base {
            let with_state = format!("{}.{}.{}", path, state.as_str(), field);
            if let Some(r) = self.component_radius(&with_state) {
                return Some(r);
            }
        }
        self.component_radius(&format!("{}.{}", path, field))
    }

    // ---- Validation ----

    pub fn validate(&self) -> Vec<String> {
        const REQUIRED: &[&str] = &[
            "background",
            "foreground",
            "card",
            "cardForeground",
            "popover",
            "popoverForeground",
            "primary",
            "primaryForeground",
            "secondary",
            "secondaryForeground",
            "muted",
            "mutedForeground",
            "accent",
            "accentForeground",
            "destructive",
            "destructiveForeground",
            "border",
            "input",
            "ring",
        ];
        let mut warnings = Vec::new();
        for key in REQUIRED {
            if !self.semantic.contains_key(*key) {
                warnings.push(format!("missing required semantic token: {key}"));
            }
        }
        if let (Some(bg), Some(fg)) = (self.color("background"), self.color("foreground")) {
            let ratio = contrast_ratio(bg, fg);
            if ratio < 4.5 {
                warnings.push(format!("foreground/background contrast {:.2} < 4.5", ratio));
            }
        }
        warnings
    }

    /// Inert fallback so components used outside a provider don't panic.
    pub fn empty_light() -> Self {
        Self {
            id: "fallback-light".into(),
            name: "Fallback Light".into(),
            appearance: Appearance::Light,
            version: "0".into(),
            author: None,
            palette_overrides: HashMap::new(),
            semantic: HashMap::new(),
            components: serde_json::Value::Null,
            radius: [("sm", 6.0), ("md", 8.0), ("lg", 10.0), ("xl", 14.0)]
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
            spacing: HashMap::new(),
            typography: TypographyConfig::default(),
            shadows: HashMap::new(),
            motion: MotionConfig::default(),
            z_index: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------
// Named semantic getters
// ---------------------------------------------------------------------

macro_rules! getters {
    ($($name:ident => $key:literal | $fallback:expr),* $(,)?) => {
        impl Theme {
            $(
                #[inline] pub fn $name(&self) -> Color {
                    self.color($key).unwrap_or($fallback)
                }
                paste! {
                    #[inline] pub fn [<$name _fill>](&self) -> Fill {
                        self.fill($key).unwrap_or(Fill::Color($fallback))
                    }
                }
            )*
        }
    };
}

getters! {
    background              => "background"              | Color::from_argb(0, 0, 0, 0),
    foreground              => "foreground"              | Color::from_rgb(0, 0, 0),
    card                    => "card"                    | Color::from_argb(0, 0, 0, 0),
    card_foreground         => "cardForeground"          | Color::from_rgb(0, 0, 0),
    popover                 => "popover"                 | Color::from_argb(0, 0, 0, 0),
    popover_foreground      => "popoverForeground"       | Color::from_rgb(0, 0, 0),
    primary                 => "primary"                 | Color::from_rgb(0, 0, 0),
    primary_foreground      => "primaryForeground"       | Color::from_rgb(255, 255, 255),
    secondary               => "secondary"               | Color::from_argb(0, 0, 0, 0),
    secondary_foreground    => "secondaryForeground"     | Color::from_rgb(0, 0, 0),
    muted                   => "muted"                   | Color::from_argb(0, 0, 0, 0),
    muted_foreground        => "mutedForeground"         | Color::from_rgb(100, 100, 100),
    accent                  => "accent"                  | Color::from_argb(0, 0, 0, 0),
    accent_foreground       => "accentForeground"        | Color::from_rgb(0, 0, 0),
    destructive             => "destructive"             | Color::from_rgb(220, 38, 38),
    destructive_foreground  => "destructiveForeground"   | Color::from_rgb(255, 255, 255),
    border                  => "border"                  | Color::from_rgb(200, 200, 200),
    input                   => "input"                   | Color::from_rgb(200, 200, 200),
    ring                    => "ring"                    | Color::from_rgb(100, 100, 255),
    disabled                => "disabled"                | Color::from_argb(0, 0, 0, 0),
    disabled_foreground     => "disabledForeground"      | Color::from_rgb(150, 150, 150),
    sidebar                 => "sidebar"                 | Color::from_argb(0, 0, 0, 0),
    sidebar_foreground      => "sidebarForeground"       | Color::from_rgb(0, 0, 0),
    sidebar_primary         => "sidebarPrimary"          | Color::from_rgb(0, 0, 0),
    sidebar_primary_foreground => "sidebarPrimaryForeground" | Color::from_rgb(255, 255, 255),
    sidebar_accent          => "sidebarAccent"           | Color::from_argb(0, 0, 0, 0),
    sidebar_accent_foreground => "sidebarAccentForeground" | Color::from_rgb(0, 0, 0),
    sidebar_border          => "sidebarBorder"           | Color::from_rgb(200, 200, 200),
    sidebar_ring            => "sidebarRing"             | Color::from_rgb(100, 100, 255),
    chart_1                 => "chart1"                  | Color::from_rgb(120, 120, 120),
    chart_2                 => "chart2"                  | Color::from_rgb(120, 120, 120),
    chart_3                 => "chart3"                  | Color::from_rgb(120, 120, 120),
    chart_4                 => "chart4"                  | Color::from_rgb(120, 120, 120),
    chart_5                 => "chart5"                  | Color::from_rgb(120, 120, 120),
}

// ---------------------------------------------------------------------
// Palette lookup — uses the functions exported by `colors.rs`
// ---------------------------------------------------------------------

fn lookup_palette_styled(path: &str) -> Option<StyledColor> {
    let (group, step) = path.split_once('.')?;

    match (group, step) {
        ("white", _) => return Some(crate::styling::colors::white()),
        ("black", _) => return Some(crate::styling::colors::black()),
        _ => {}
    }

    macro_rules! try_family {
        ($fam:ident) => {
            paste! {
                match step {
                    "50"  => Some(crate::styling::colors::[<$fam _ 50>]()),
                    "100" => Some(crate::styling::colors::[<$fam _ 100>]()),
                    "200" => Some(crate::styling::colors::[<$fam _ 200>]()),
                    "300" => Some(crate::styling::colors::[<$fam _ 300>]()),
                    "400" => Some(crate::styling::colors::[<$fam _ 400>]()),
                    "500" => Some(crate::styling::colors::[<$fam _ 500>]()),
                    "600" => Some(crate::styling::colors::[<$fam _ 600>]()),
                    "700" => Some(crate::styling::colors::[<$fam _ 700>]()),
                    "800" => Some(crate::styling::colors::[<$fam _ 800>]()),
                    "900" => Some(crate::styling::colors::[<$fam _ 900>]()),
                    "950" => Some(crate::styling::colors::[<$fam _ 950>]()),
                    _ => None,
                }
            }
        };
    }

    match group {
        "stone" => try_family!(stone),
        "mauve" => try_family!(mauve),
        "olive" => try_family!(olive),
        "mist" => try_family!(mist),
        "taupe" => try_family!(taupe),
        "slate" => try_family!(slate),
        "gray" => try_family!(gray),
        "zinc" => try_family!(zinc),
        "neutral" => try_family!(neutral),
        "red" => try_family!(red),
        "orange" => try_family!(orange),
        "amber" => try_family!(amber),
        "yellow" => try_family!(yellow),
        "lime" => try_family!(lime),
        "green" => try_family!(green),
        "emerald" => try_family!(emerald),
        "teal" => try_family!(teal),
        "cyan" => try_family!(cyan),
        "sky" => try_family!(sky),
        "blue" => try_family!(blue),
        "indigo" => try_family!(indigo),
        "violet" => try_family!(violet),
        "purple" => try_family!(purple),
        "fuchsia" => try_family!(fuchsia),
        "pink" => try_family!(pink),
        "rose" => try_family!(rose),
        _ => None,
    }
}

// ---------------------------------------------------------------------
// Resolution helpers
// ---------------------------------------------------------------------

fn parse_hex(s: &str) -> Result<Color, ThemeError> {
    let s = s.trim().trim_start_matches('#');
    let expand = |h: &str| -> String {
        let mut out = String::with_capacity(h.len() * 2);
        for ch in h.chars() {
            out.push(ch);
            out.push(ch);
        }
        out
    };
    let (rgb, alpha) = match s.len() {
        3 | 4 => {
            let a = if s.len() == 4 {
                u8::from_str_radix(&expand(&s[3..]), 16)
                    .map_err(|_| ThemeError::InvalidHex(s.into()))?
            } else {
                255
            };
            (expand(&s[..3]), a)
        }
        6 | 8 => {
            let a = if s.len() == 8 {
                u8::from_str_radix(&s[6..8], 16).map_err(|_| ThemeError::InvalidHex(s.into()))?
            } else {
                255
            };
            (s[..6].to_string(), a)
        }
        _ => return Err(ThemeError::InvalidHex(s.into())),
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).map_err(|_| ThemeError::InvalidHex(s.into()))?;
    let g = u8::from_str_radix(&rgb[2..4], 16).map_err(|_| ThemeError::InvalidHex(s.into()))?;
    let b = u8::from_str_radix(&rgb[4..6], 16).map_err(|_| ThemeError::InvalidHex(s.into()))?;
    Ok(Color::from_argb(alpha, r, g, b))
}

fn apply_alpha_pct(c: Color, pct: Option<f32>) -> Color {
    match pct {
        Some(p) => {
            let a = (p.clamp(0.0, 100.0) / 100.0 * 255.0).round() as u8;
            c.with_a(a)
        }
        None => c,
    }
}

fn with_alpha_pct(styled: StyledColor, pct: Option<f32>) -> Result<Color, ThemeError> {
    let styled = match pct {
        Some(p) => styled.with_opacity((p / 100.0).clamp(0.0, 1.0)),
        None => styled,
    };
    styled
        .to_color()
        .ok_or_else(|| ThemeError::NotASolidColor("gradient".into()))
}

/// Resolve a single color token (with optional `/NN` suffix).
pub fn resolve_color(
    s: &str,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<Color, ThemeError> {
    let s = s.trim();

    if s.eq_ignore_ascii_case("transparent") {
        return Ok(Color::from_argb(0, 0, 0, 0));
    }

    if let Some(inner) = s.strip_prefix('{').and_then(|x| x.strip_suffix('}')) {
        let (path, alpha_pct) = match inner.split_once('/') {
            Some((p, a)) => (
                p,
                Some(
                    a.parse::<f32>()
                        .map_err(|_| ThemeError::InvalidPaletteRef(s.into()))?,
                ),
            ),
            None => (inner, None),
        };

        // `{semantic.foo}` — already-resolved
        if let Some(key) = path.strip_prefix("semantic.") {
            let base = match semantic.get(key) {
                Some(Fill::Color(c)) => *c,
                _ => return Err(ThemeError::NotASolidColor(path.into())),
            };
            return Ok(apply_alpha_pct(base, alpha_pct));
        }

        // User palette override
        if let Some(c) = overrides.get(path) {
            return Ok(apply_alpha_pct(*c, alpha_pct));
        }

        // Built-in palette from `colors.rs`
        let styled = lookup_palette_styled(path)
            .ok_or_else(|| ThemeError::UnknownPaletteStep(path.into()))?;
        return with_alpha_pct(styled, alpha_pct);
    }

    parse_hex(s)
}

fn build_stops(
    stops: &[Stop],
    overrides: &HashMap<String, Color>,
) -> Result<Vec<(Color, f32)>, ThemeError> {
    stops
        .iter()
        .map(|s| Ok((resolve_color(&s.color, overrides, &HashMap::new())?, s.at)))
        .collect()
}

pub fn resolve_fill(
    token: &TokenValue,
    overrides: &HashMap<String, Color>,
    semantic: &HashMap<String, Fill>,
) -> Result<Fill, ThemeError> {
    match token {
        TokenValue::Color(s) => Ok(Fill::Color(resolve_color(s, overrides, semantic)?)),
        TokenValue::Linear { linear } => {
            let mut g = LinearGradient::new().angle(linear.angle);
            for (color, at) in build_stops(&linear.stops, overrides)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
        TokenValue::Radial { radial } => {
            // Freya's `RadialGradient` only accepts stops; `center` /
            // `radius` from the schema are kept for forward compatibility.
            let _ = (radial.center, radial.radius);
            let mut g = RadialGradient::new();
            for (color, at) in build_stops(&radial.stops, overrides)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
        TokenValue::Conic { conic } => {
            // `center` is ignored: Freya's `ConicGradient` only accepts
            // `.angle(..)` and `.stop(..)`.
            let _ = conic.center;
            let mut g = ConicGradient::new().angle(conic.angle);
            for (color, at) in build_stops(&conic.stops, overrides)? {
                g = g.stop((color, at));
            }
            Ok(Fill::from(g))
        }
    }
}

fn contrast_ratio(a: Color, b: Color) -> f32 {
    fn lum(c: Color) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }
    let (l1, l2) = {
        let x = lum(a);
        let y = lum(b);
        if x > y { (x, y) } else { (y, x) }
    };
    (l1 + 0.05) / (l2 + 0.05)
}

// ---------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------

#[derive(Default)]
pub struct ThemeRegistry {
    themes: RwLock<HashMap<String, Arc<Theme>>>,
    active_id: RwLock<Option<String>>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self, json: &str) -> Result<Arc<Theme>, ThemeError> {
        let cfg: ThemeConfig = serde_json::from_str(json)?;
        let theme = Arc::new(Theme::from_config(cfg)?);
        for w in theme.validate() {
            eprintln!("theme '{}': {}", theme.id, w);
        }
        let id = theme.id.clone();
        self.themes
            .write()
            .unwrap()
            .insert(id.clone(), theme.clone());
        let mut active = self.active_id.write().unwrap();
        if active.is_none() {
            *active = Some(id);
        }
        Ok(theme)
    }

    pub fn load_dir(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        let mut count = 0usize;
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let s = std::fs::read_to_string(&path)?;
                self.load(&s)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Re-read every `.json` in `dir` and update the registry in place.
    /// Removed files are not pruned — call `remove(id)` first if needed.
    pub fn reload_dir(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        self.load_dir(dir)
    }

    pub fn get(&self, id: &str) -> Option<Arc<Theme>> {
        self.themes.read().unwrap().get(id).cloned()
    }

    pub fn active(&self) -> Option<Arc<Theme>> {
        let id = self.active_id.read().unwrap().clone()?;
        self.get(&id)
    }

    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        if !self.themes.read().unwrap().contains_key(id) {
            return Err(ThemeError::UnknownTheme(id.into()));
        }
        *self.active_id.write().unwrap() = Some(id.into());
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Option<Arc<Theme>> {
        let t = self.themes.write().unwrap().remove(id);
        if let Some(t) = &t {
            let mut active = self.active_id.write().unwrap();
            if active.as_deref() == Some(t.id.as_str()) {
                *active = None;
            }
        }
        t
    }

    pub fn list(&self) -> Vec<Arc<Theme>> {
        self.themes.read().unwrap().values().cloned().collect()
    }

    pub fn ids(&self) -> Vec<String> {
        self.themes.read().unwrap().keys().cloned().collect()
    }

    pub fn from_json(json: &str) -> Result<Theme, ThemeError> {
        Theme::from_json_str(json)
    }

    pub fn from_dir(dir: &std::path::Path) -> Result<Self, ThemeError> {
        let registry = Self::new();
        registry.load_dir(dir)?;
        Ok(registry)
    }

    pub fn insert(&self, theme: Theme) -> Arc<Theme> {
        let arc = Arc::new(theme);
        self.themes
            .write()
            .unwrap()
            .insert(arc.id.clone(), arc.clone());
        let mut active = self.active_id.write().unwrap();
        if active.is_none() {
            *active = Some(arc.id.clone());
        }
        arc
    }

    pub fn reload(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        self.reload_dir(dir)
    }
}

// ---------------------------------------------------------------------
// Freya context integration
// ---------------------------------------------------------------------

/// Reactive handle to the active theme. `active` is Freya's
/// `State<Arc<Theme>>` — the same type returned by [`use_cn_theme`].
#[derive(Clone)]
pub struct ThemeHandle {
    pub registry: Arc<ThemeRegistry>,
    pub active: State<Arc<Theme>>,
}

impl ThemeHandle {
    pub fn set_active(&self, id: &str) -> Result<(), ThemeError> {
        self.registry.set_active(id)?;
        if let Some(t) = self.registry.active() {
            set_active_theme(t.clone()); // ← new
            let mut active = self.active;
            active.set(t);
        }
        Ok(())
    }

    pub fn reload_dir(&self, dir: &std::path::Path) -> Result<usize, ThemeError> {
        let n = self.registry.reload_dir(dir)?;
        let prev_id = self.active.read().id.clone();
        if let Some(t) = self.registry.get(&prev_id) {
            set_active_theme(t.clone()); // ← new
            let mut active = self.active;
            active.set(t);
        }
        Ok(n)
    }
}

pub fn load_theme_registry(initial_json: &str) -> ThemeHandle {
    use_hook(|| {
        let registry = Arc::new(ThemeRegistry::new());
        let theme = registry.load(initial_json).expect("invalid initial theme");
        set_active_theme(theme.clone()); // ← new
        let active = State::create(theme);
        provide_context(active);
        ThemeHandle { registry, active }
    })
}

pub fn load_theme_registry_from_dir(dir: &std::path::Path) -> ThemeHandle {
    use_hook(|| {
        let registry = Arc::new(ThemeRegistry::new());
        registry
            .load_dir(dir)
            .expect("failed to load theme directory");
        let active = registry.active().expect("no theme was loaded");
        set_active_theme(active.clone()); // ← new
        let state = State::create(active);
        provide_context(state);
        ThemeHandle {
            registry,
            active: state,
        }
    })
}

/// Subscribe to the currently active theme.
pub fn use_cn_theme() -> State<Arc<Theme>> {
    use_consume::<State<Arc<Theme>>>()
}

/// Convenience: clone the active theme out of the reactive state.
pub fn use_theme() -> Arc<Theme> {
    use_cn_theme().read().clone()
}

/// Fallback to a light, empty theme when none is provided.
pub fn get_cn_theme_or_default() -> Readable<Arc<Theme>> {
    try_consume_context::<State<Arc<Theme>>>()
        .map(|s| s.into())
        .unwrap_or_else(|| Arc::new(Theme::empty_light()).into())
}
