use serde::Serialize;

/// Supported output formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Yaml,
    Markdown,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Human => write!(f, "human"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Unknown output format: {s}")),
        }
    }
}

/// Formats serializable data into the requested output format.
pub struct OutputFormatter {
    pub format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Format a serializable value into a string.
    pub fn format<T: Serialize>(&self, value: &T) -> crate::error::Result<String> {
        match self.format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(value).map_err(crate::error::TomeError::from)
            }
            other => {
                tracing::warn!(
                    format = %other,
                    "Requested output format is not yet implemented, falling back to JSON"
                );
                serde_json::to_string_pretty(value).map_err(crate::error::TomeError::from)
            }
        }
    }
}
