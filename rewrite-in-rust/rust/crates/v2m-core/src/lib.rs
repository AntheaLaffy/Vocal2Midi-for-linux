//! Shared control-plane primitives for the Vocal2Midi Rust rewrite.
//!
//! This crate is intentionally not wired into the Python runtime. It gives the
//! rewrite workspace a small independently testable Rust surface before any
//! business migration unit is promoted.

pub mod application;
pub mod batch_cli_planning;
pub mod batch_cli_reslice_json;
pub mod device;
pub mod download_models_archive;
pub mod download_models_catalog;
pub mod download_models_cli;
pub mod download_models_effectful;
pub mod export;
pub mod game;
pub mod ja_g2p;
pub mod lyric_matching_file;
pub mod lyric_sequence;
pub mod midi_export;
pub mod quant;
pub mod slice_bounds;
pub mod slice_method;
pub mod slicer_default;
pub mod slicer_grid;
pub mod slicer_heuristic;
pub mod slicer_pitch;
pub mod slicer_segment;
pub mod slicer_window;
pub mod ustx_pitch_curve;
pub mod ustx_project;
pub mod web_config;
pub mod web_filesystem_picker;
pub mod web_model_download;
pub mod web_model_download_execution;
pub mod web_model_download_lifecycle;
pub mod web_model_download_process;
pub mod web_model_download_termination;
pub mod web_output_download;
pub mod web_pipeline_events;
pub mod web_settings;
pub mod web_stream;
pub mod web_task;
pub mod zh_g2p;

/// Manifest states accepted by `rewrite-in-rust/manifest.yaml`.
pub const STATUS_VALUES: &[&str] = &[
    "planned",
    "active",
    "reimplemented",
    "verified",
    "promoted",
    "optimized",
    "blocked",
];

/// Migration status for one independently verifiable rewrite unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    Planned,
    Active,
    Reimplemented,
    Verified,
    Promoted,
    Optimized,
    Blocked,
}

impl MigrationStatus {
    /// Returns the string value used in the YAML manifest.
    pub const fn as_manifest_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Reimplemented => "reimplemented",
            Self::Verified => "verified",
            Self::Promoted => "promoted",
            Self::Optimized => "optimized",
            Self::Blocked => "blocked",
        }
    }

    /// Parses a manifest status value.
    pub fn from_manifest_str(value: &str) -> Option<Self> {
        match value {
            "planned" => Some(Self::Planned),
            "active" => Some(Self::Active),
            "reimplemented" => Some(Self::Reimplemented),
            "verified" => Some(Self::Verified),
            "promoted" => Some(Self::Promoted),
            "optimized" => Some(Self::Optimized),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }

    /// Returns true when this status is a runtime-owner promotion state.
    pub const fn is_runtime_owner_state(self) -> bool {
        matches!(self, Self::Promoted | Self::Optimized)
    }
}

/// Returns true when a status value is accepted by the rewrite manifest.
pub fn is_known_status(value: &str) -> bool {
    MigrationStatus::from_manifest_str(value).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_values_round_trip() {
        for value in STATUS_VALUES {
            let status = MigrationStatus::from_manifest_str(value).unwrap();
            assert_eq!(status.as_manifest_str(), *value);
        }
    }

    #[test]
    fn unknown_status_is_rejected() {
        assert!(!is_known_status("done"));
        assert!(!is_known_status(""));
        assert!(!is_known_status("verified "));
    }

    #[test]
    fn only_promoted_states_own_runtime() {
        assert!(!MigrationStatus::Planned.is_runtime_owner_state());
        assert!(!MigrationStatus::Verified.is_runtime_owner_state());
        assert!(MigrationStatus::Promoted.is_runtime_owner_state());
        assert!(MigrationStatus::Optimized.is_runtime_owner_state());
    }
}
