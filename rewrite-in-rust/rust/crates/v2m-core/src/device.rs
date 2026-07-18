//! Runtime device name normalization.
//!
//! This module mirrors `inference/device_utils.py::normalize_runtime_device`
//! without changing the Python runtime owner.

/// Runtime platform used to resolve the implicit default device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePlatform {
    /// Represents the Python-compatible windows case.
    Windows,
    /// Represents the Python-compatible other case.
    Other,
}

impl RuntimePlatform {
    /// Returns the default device for this platform.
    pub const fn default_device(self) -> &'static str {
        match self {
            Self::Windows => "dml",
            Self::Other => "cpu",
        }
    }
}

/// Returns the current compilation target's runtime platform.
pub const fn current_runtime_platform() -> RuntimePlatform {
    if cfg!(target_os = "windows") {
        RuntimePlatform::Windows
    } else {
        RuntimePlatform::Other
    }
}

/// Normalizes a device name using the current compilation target's default.
pub fn normalize_runtime_device(device: Option<&str>) -> String {
    normalize_runtime_device_for_platform(device, current_runtime_platform())
}

/// Normalizes a device name using an explicit platform default.
pub fn normalize_runtime_device_for_platform(
    device: Option<&str>,
    platform: RuntimePlatform,
) -> String {
    normalize_runtime_device_with_default(device, platform.default_device())
}

/// Normalizes a device name using an explicit default value.
pub fn normalize_runtime_device_with_default(device: Option<&str>, default: &str) -> String {
    let raw_value = match device {
        Some(value) if !value.is_empty() => value,
        _ => default,
    };
    let mut value = raw_value.trim().to_lowercase();
    if value.is_empty() {
        value = default.to_string();
    }

    match value.as_str() {
        "" | "cuda" | "directml" | "dml" | "gpu" => "dml".to_string(),
        "cpu" => "cpu".to_string(),
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = include_str!("../../../../fixtures/runtime_device_normalization.tsv");

    fn parse_platform(value: &str) -> RuntimePlatform {
        match value {
            "windows" => RuntimePlatform::Windows,
            "other" => RuntimePlatform::Other,
            _ => panic!("unknown platform {value}"),
        }
    }

    fn parse_optional_value(value: &'static str) -> Option<&'static str> {
        match value {
            "__none__" => None,
            "__empty__" => Some(""),
            "__space__" => Some("   "),
            "__padded_directml__" => Some(" DirectML "),
            "__padded_unknown__" => Some(" Metal "),
            _ => Some(value),
        }
    }

    #[test]
    fn device_normalization_follows_parity_fixture_table() {
        for (line_number, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let platform = parse_platform(fields.next().unwrap());
            let device = parse_optional_value(fields.next().unwrap());
            let default = parse_optional_value(fields.next().unwrap());
            let expected = fields.next().unwrap();

            let actual = match default {
                Some(default) => normalize_runtime_device_with_default(device, default),
                None => normalize_runtime_device_for_platform(device, platform),
            };
            assert_eq!(actual, expected, "fixture line {}", line_number + 1);
        }
    }

    #[test]
    fn device_normalization_applies_platform_defaults() {
        assert_eq!(
            normalize_runtime_device_for_platform(None, RuntimePlatform::Other),
            "cpu"
        );
        assert_eq!(
            normalize_runtime_device_for_platform(None, RuntimePlatform::Windows),
            "dml"
        );
    }

    #[test]
    fn device_normalization_maps_legacy_aliases_to_dml() {
        for value in ["cuda", "directml", "dml", "gpu", " CUDA ", " DirectML "] {
            assert_eq!(
                normalize_runtime_device_for_platform(Some(value), RuntimePlatform::Other),
                "dml"
            );
        }
    }

    #[test]
    fn device_normalization_preserves_unknown_values_after_trim_and_lowercase() {
        assert_eq!(
            normalize_runtime_device_for_platform(Some(" Metal "), RuntimePlatform::Other),
            "metal"
        );
        assert_eq!(
            normalize_runtime_device_for_platform(Some("vulkan"), RuntimePlatform::Other),
            "vulkan"
        );
    }

    #[test]
    fn device_normalization_preserves_python_explicit_default_edges() {
        assert_eq!(normalize_runtime_device_with_default(None, ""), "dml");
        assert_eq!(
            normalize_runtime_device_with_default(Some("   "), ""),
            "dml"
        );
        assert_eq!(
            normalize_runtime_device_with_default(Some(""), "cpu"),
            "cpu"
        );
    }
}
