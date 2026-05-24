use std::path::Path;

use serde::Deserialize;

fn default_piper_sample_rate() -> u32 {
    22050
}

/// Application configuration, loaded from a JSON file.
///
/// All runtime values except `VIKUNJA_TOKEN` live here; the token stays in the
/// environment so it can be kept out of the config file and source control.
#[derive(Deserialize)]
pub struct Config {
    /// Path to the Vosk model directory.
    pub vosk_model: String,
    /// Base URL of the Vikunja instance; trailing slashes are stripped on load.
    pub vikunja_url: String,
    /// Vikunja project ID for reminder tasks.
    pub vikunja_project_id: u64,
    /// Vikunja project ID for shopping list tasks.
    pub vikunja_shopping_project_id: u64,
    /// Path to the Piper TTS binary. A bare name (no `/`) is resolved via `PATH`.
    pub piper_bin: String,
    /// Path to the Piper `.onnx` voice model file.
    pub piper_model: String,
    /// Sample rate of the Piper model in Hz. Defaults to `22050` if omitted.
    #[serde(default = "default_piper_sample_rate")]
    pub piper_sample_rate: u32,
}

impl Config {
    /// Reads the JSON file at `path`, parses it, normalises fields, and validates.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read config file '{}': {}", path, e))?;
        let mut config: Self = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Invalid config file '{}': {}", path, e))?;
        config.vikunja_url = config.vikunja_url.trim_end_matches('/').to_string();
        config.validate()?;
        Ok(config)
    }

    /// Returns an error if any required path is missing or any field is invalid.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !Path::new(&self.vosk_model).is_dir() {
            anyhow::bail!(
                "`vosk_model` '{}' does not exist or is not a directory",
                self.vosk_model
            );
        }
        // Only check piper_bin on the filesystem if it looks like a path.
        // A bare name (e.g. "piper") is assumed to be on PATH.
        if self.piper_bin.contains('/') && !Path::new(&self.piper_bin).is_file() {
            anyhow::bail!("`piper_bin` '{}' does not exist", self.piper_bin);
        }
        if !Path::new(&self.piper_model).is_file() {
            anyhow::bail!(
                "`piper_model` '{}' does not exist or is not a file",
                self.piper_model
            );
        }
        if self.vikunja_url.is_empty() {
            anyhow::bail!("`vikunja_url` must not be empty");
        }
        Ok(())
    }
}
