use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeasuredSceneDto {
    Live,
    WholeGame,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredAssessmentDto {
    pub eligible: bool,
    pub needs_five_pairs: bool,
    pub reasons: Vec<String>,
    pub pair_ratios: Vec<f64>,
    pub median_ratio: Option<f64>,
    pub sample_cv: Option<f64>,
    pub experimental: bool,
}

/// Durable evidence, not an executable command or a mutable Run configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedMeasuredSceneDto {
    pub report: serde_json::Value,
    pub profile_revision: String,
    pub policy_revision: u64,
    pub accepted_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MeasuredScenesDto {
    pub revision: u64,
    pub live: Option<AcceptedMeasuredSceneDto>,
    pub whole_game: Option<AcceptedMeasuredSceneDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredReviewDto {
    pub operation_id: String,
    pub generation: u64,
    pub target_id: String,
    pub target_name: String,
    pub scene: MeasuredSceneDto,
    pub profile_revision: String,
    pub policy_revision: u64,
    pub overlay_revision: u64,
    pub baseline_parameters: BTreeMap<String, u32>,
    pub candidate_parameters: BTreeMap<String, u32>,
    pub assessment: MeasuredAssessmentDto,
    pub qualification_reasons: Vec<String>,
    pub cold_verification_seconds: f64,
    pub report: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMeasurementRequestDto {
    pub target_id: String,
    pub scene: MeasuredSceneDto,
    pub candidate_parameters: BTreeMap<String, u32>,
    pub pairs: u32,
    pub budget: u32,
    pub fixture_sgf: String,
    pub replace_current_document: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMeasurementProbeDto {
    pub operation_id: String,
    pub row_id: String,
    pub samples_seconds: Vec<f64>,
    pub sample_times_ms: Vec<[f64; 2]>,
    pub finished: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMeasurementProgressDto {
    pub operation_id: String,
    pub row_id: String,
    pub scene: MeasuredSceneDto,
    pub round: u32,
    pub candidate: bool,
    pub sampling: bool,
    pub closing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMeasurementResultDto {
    pub report: serde_json::Value,
    pub evidence: serde_json::Value,
}
