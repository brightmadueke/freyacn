use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: String,
    pub registry: Registry,
    pub paths: Paths,
    pub aliases: Aliases,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub icons: Icons,
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Debug, Deserialize)]
pub struct Registry {
    pub url: String,
    pub fallback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub components: PathBuf,
    pub module_root: PathBuf,
    pub theme: PathBuf,
    pub utils: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Aliases {
    pub components: String,
    pub theme: String,
    pub utils: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Theme {
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default = "default_true")]
    pub compile_time: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Icons {
    #[serde(default = "default_icons")]
    pub library: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Style {
    #[serde(default = "default_style")]
    pub preset: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Dependencies {
    #[serde(default)]
    pub required: Vec<String>,
}

fn default_preset() -> String { "neutral".into() }
fn default_icons() -> String { "freyacn-icons".into() }
fn default_style() -> String { "freyacn-default".into() }
fn default_true() -> bool { true }

pub fn load() -> anyhow::Result<Config> {
    let text = std::fs::read_to_string("freyacn.toml")?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}