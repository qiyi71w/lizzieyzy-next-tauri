use serde::{Deserialize, Serialize};

/// Outer origin is physical; client dimensions are logical at the recorded scale.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowGeometryDto {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub maximized: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometryStatusDto {
    pub phase: String,
    pub geometry: Option<WindowGeometryDto>,
    pub error: Option<String>,
}
