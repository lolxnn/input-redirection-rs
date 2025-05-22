use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub target_ip: String,
    pub invert_lx: bool,
    pub invert_ly: bool,
    pub invert_rx: bool,
    pub invert_ry: bool,
    pub deadzone_lstick: f32,
    pub deadzone_rstick: f32,
}

// Default values for the config
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_ip: "0.0.0.0".into(),
            invert_lx: false,
            invert_ly: false,
            invert_rx: false,
            invert_ry: false,
            deadzone_lstick: 0.10, // 10%
            deadzone_rstick: 0.10, // 10%
        }
    }
}

fn config_path() -> io::Result<PathBuf> {
    ProjectDirs::from("com", "Rust3DSInputRedirector", "Rust3DSInputRedirector")
        .map(|d| d.config_dir().join("config.toml"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not determine config dir"))
}

impl AppConfig {
    pub fn load() -> io::Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let txt = fs::read_to_string(&path)?;
        toml::from_str(&txt).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("TOML parse error: {e}"))
        })
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let toml =
            toml::to_string_pretty(self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(path, toml)
    }
}
