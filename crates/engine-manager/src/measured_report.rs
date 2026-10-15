//! Frozen application/totalGpu v1 import. Report strings are data, never launch inputs.
pub mod fingerprint;
pub mod measurement;
use app_model::{MeasuredAssessmentDto, MeasuredSceneDto};
use serde::de::{MapAccess, Visitor};
use serde::Deserializer;
use serde_json::{value::RawValue, Value};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

pub const MAX_REPORT_BYTES: usize = 1024 * 1024;
pub const LIVE_KEYS: [&str; 2] = ["numSearchThreads", "nnMaxBatchSize"];
pub const WHOLE_KEYS: [&str; 3] = [
    "numAnalysisThreads",
    "numSearchThreadsPerAnalysisThread",
    "nnMaxBatchSize",
];

#[derive(Debug, Clone)]
pub struct Report {
    pub raw: Value,
    pub scene: MeasuredSceneDto,
    pub baseline: BTreeMap<String, u32>,
    pub candidate: BTreeMap<String, u32>,
    pub budget: u32,
    pub positions: u32,
    rows: Vec<Row>,
    physical_memory: f64,
}
#[derive(Debug, Clone)]
struct Row {
    candidate: bool,
    round: u32,
    seconds: f64,
    response: f64,
    event_loop: f64,
    memory: f64,
    processes: u32,
    first: Option<f64>,
    visits: Vec<u32>,
}

/// Validate duplicate object members before serde_json's ordinary last-wins conversion.
struct Members;
impl<'de> Visitor<'de> for Members {
    type Value = Vec<(String, &'de RawValue)>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("JSON object")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut seen = HashSet::new();
        let mut values = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, &RawValue>()? {
            if !seen.insert(key.clone()) {
                return Err(serde::de::Error::custom("duplicate report key"));
            }
            values.push((key, value));
        }
        Ok(values)
    }
}
fn unique(raw: &RawValue, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("report nesting exceeds 64".into());
    }
    match raw.get().as_bytes().first() {
        Some(b'{') => {
            let mut de = serde_json::Deserializer::from_str(raw.get());
            for (_, child) in de.deserialize_map(Members).map_err(|e| e.to_string())? {
                unique(child, depth + 1)?;
            }
        }
        Some(b'[') => {
            for child in serde_json::from_str::<Vec<&RawValue>>(raw.get()).map_err(|e| e.to_string())? {
                unique(child, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value.get(key).ok_or_else(|| format!("missing {key}"))
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    field(value, key)?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("{key} must be a nonblank string"))
}
fn hash(value: &Value, key: &str) -> Result<(), String> {
    let s = text(value, key)?;
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(format!("invalid {key} SHA256"));
    }
    Ok(())
}
/// Decimal arithmetic on the original JSON number; never round through a double.
pub fn exact_count(value: &Value, minimum: u32) -> Result<u32, String> {
    let number = value.as_number().ok_or("expected JSON number")?.to_string();
    let (mantissa, exponent) = number.split_once(['e', 'E']).unwrap_or((&number, "0"));
    let exponent = exponent
        .parse::<i64>()
        .map_err(|_| "integer exponent out of range")?;
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{whole}{fraction}");
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return if minimum == 0 {
            Ok(0)
        } else {
            Err("integer below minimum".into())
        };
    }
    if negative {
        return Err("negative count".into());
    }
    let shift = exponent
        .checked_sub(fraction.len() as i64)
        .ok_or("integer exponent out of range")?;
    let significant = if shift < 0 {
        let remove = shift.unsigned_abs();
        if remove >= digits.len() as u64 {
            return Err("nonintegral count".into());
        }
        let split = digits.len() - remove as usize;
        if !digits[split..].bytes().all(|b| b == b'0') {
            return Err("nonintegral count".into());
        }
        &digits[..split]
    } else {
        digits
    };
    if significant.len() as i64 + shift.max(0) > 10 {
        return Err("count exceeds i32".into());
    }
    let mut result = significant.parse::<u64>().map_err(|_| "invalid count")?;
    if shift > 0 {
        result *= 10u64.pow(shift as u32);
    }
    if result < minimum as u64 || result > i32::MAX as u64 {
        return Err("count outside i32 range".into());
    }
    Ok(result as u32)
}
fn count(v: &Value, key: &str, min: u32) -> Result<u32, String> {
    exact_count(field(v, key)?, min).map_err(|e| format!("{key}: {e}"))
}
fn real(v: &Value, key: &str, positive: bool) -> Result<f64, String> {
    let n = field(v, key)?
        .as_f64()
        .ok_or_else(|| format!("{key} must be numeric"))?;
    if !n.is_finite() || n < 0.0 || (positive && n == 0.0) {
        return Err(format!("{key} outside finite range"));
    }
    Ok(n)
}
pub fn parameters(v: &Value, scene: MeasuredSceneDto) -> Result<BTreeMap<String, u32>, String> {
    let map = v.as_object().ok_or("parameters must be an object")?;
    let keys: &[&str] = match scene {
        MeasuredSceneDto::Live => &LIVE_KEYS,
        MeasuredSceneDto::WholeGame => &WHOLE_KEYS,
    };
    if map.len() != keys.len() || map.keys().any(|k| !keys.contains(&k.as_str())) {
        return Err("parameters require the exact scene key set".into());
    }
    keys.iter()
        .map(|key| {
            let n = count(v, key, 1)?;
            if n > if *key == "nnMaxBatchSize" { 65536 } else { 4096 } {
                return Err(format!("{key} outside parameter range"));
            }
            Ok((key.to_string(), n))
        })
        .collect()
}
pub fn parse(bytes: &[u8]) -> Result<Report, String> {
    if bytes.len() > MAX_REPORT_BYTES {
        return Err("report exceeds 1 MiB".into());
    }
    let raw: &RawValue = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    unique(raw, 0)?;
    let v: Value = serde_json::from_str(raw.get()).map_err(|e| e.to_string())?;
    if count(&v, "schemaVersion", 1)? != 1 {
        return Err("unsupported schemaVersion".into());
    }
    let scene = match text(&v, "scene")? {
        "live" => MeasuredSceneDto::Live,
        "whole-game" => MeasuredSceneDto::WholeGame,
        _ => return Err("unsupported scene".into()),
    };
    if text(&v, "measurementMode")? != "application" || text(&v, "metricScope")? != "totalGpu" {
        return Err("requires application/totalGpu measurement".into());
    }
    let fp = field(&v, "fingerprint")?;
    if !fp.is_object() {
        return Err("fingerprint must be an object".into());
    }
    for key in ["engineSha256", "modelSha256", "configSha256"] {
        hash(fp, key)?;
    }
    for item in field(fp, "configIncludes")?
        .as_array()
        .ok_or("configIncludes must be an array")?
    {
        hash(&serde_json::json!({"include":item}), "include")?;
    }
    text(fp, "gpuName")?;
    text(fp, "driverVersion")?;
    let physical_memory = real(fp, "memoryMiB", true)?;
    if !field(fp, "commandSemantics")?.is_object() {
        return Err("commandSemantics must be an object".into());
    }
    hash(&v, "fixtureSha256")?;
    let budget = count(&v, "budget", 1)?;
    let positions = count(&v, "positions", 1)?;
    if scene == MeasuredSceneDto::Live && positions != 1 {
        return Err("live positions must equal 1".into());
    }
    let baseline = parameters(field(&v, "baselineParameters")?, scene)?;
    let candidate = parameters(field(&v, "candidateParameters")?, scene)?;
    let mut rows = Vec::new();
    for row in field(&v, "runs")?.as_array().ok_or("runs must be an array")? {
        let candidate = match text(row, "profile")? {
            "baseline" => false,
            "candidate" => true,
            _ => return Err("invalid run profile".into()),
        };
        if text(row, "phase")? != "warm" {
            return Err("runs must contain only warm rows".into());
        }
        let seconds = real(row, "seconds", true)?;
        let (first, visits) = match scene {
            MeasuredSceneDto::Live => {
                if row.get("rootVisitsByTurn").is_some() {
                    return Err("live forbids rootVisitsByTurn".into());
                }
                let first = real(row, "firstResultSeconds", true)?;
                if first > seconds {
                    return Err("first result follows completion".into());
                }
                (Some(first), vec![count(row, "observedRootVisits", 0)?])
            }
            MeasuredSceneDto::WholeGame => {
                if row.get("firstResultSeconds").is_some() || row.get("observedRootVisits").is_some() {
                    return Err("whole-game forbids live fields".into());
                }
                let visits = field(row, "rootVisitsByTurn")?
                    .as_array()
                    .ok_or("rootVisitsByTurn must be an array")?;
                if visits.len() != positions as usize {
                    return Err("rootVisitsByTurn length differs from positions".into());
                }
                (
                    None,
                    visits
                        .iter()
                        .map(|v| exact_count(v, 0))
                        .collect::<Result<_, _>>()?,
                )
            }
        };
        rows.push(Row {
            candidate,
            round: count(row, "round", 1)?,
            seconds,
            response: real(row, "responseSeconds", false)?,
            event_loop: real(row, "edtP95Seconds", false)?,
            memory: real(row, "maxMemoryMiB", true)?,
            processes: count(row, "maxEngineProcesses", 0)?,
            first,
            visits,
        });
    }
    Ok(Report {
        raw: v,
        scene,
        baseline,
        candidate,
        budget,
        positions,
        rows,
        physical_memory,
    })
}
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
fn maximum(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max)
}
fn cv(values: &[f64]) -> f64 {
    let scale = maximum(values);
    let mean = values.iter().map(|v| v / scale).sum::<f64>() / values.len() as f64;
    (values.iter().map(|v| (v / scale - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt() / mean
}
impl Report {
    pub fn assess(&self) -> MeasuredAssessmentDto {
        let mut out = MeasuredAssessmentDto {
            eligible: false,
            needs_five_pairs: false,
            reasons: Vec::new(),
            pair_ratios: Vec::new(),
            median_ratio: None,
            sample_cv: None,
            experimental: true,
        };
        if self.baseline == self.candidate {
            out.reasons
                .push("Baseline and candidate parameters are identical".into());
        }
        if !matches!(self.rows.len(), 6 | 10) {
            out.reasons
                .push("Exactly three or five warm pairs are required".into());
            return out;
        }
        for (i, pair) in self.rows.chunks_exact(2).enumerate() {
            if pair[0].round != i as u32 + 1
                || pair[1].round != i as u32 + 1
                || pair[0].candidate == pair[1].candidate
                || (i > 0 && pair[0].candidate == self.rows[2 * i - 2].candidate)
            {
                out.reasons.push(
                    "Pairs must preserve independent alternating execution order and consecutive rounds"
                        .into(),
                );
                return out;
            }
            let (b, c) = if pair[0].candidate {
                (&pair[1], &pair[0])
            } else {
                (&pair[0], &pair[1])
            };
            let ratio = b.seconds / c.seconds;
            if !ratio.is_finite() || ratio <= 1.0 {
                out.reasons
                    .push(format!("Pair {} does not improve completion", i + 1));
            }
            if ratio.is_finite() {
                out.pair_ratios.push(ratio);
            }
        }
        if out.pair_ratios.len() == self.rows.len() / 2 {
            let ratio = median(out.pair_ratios.clone());
            out.median_ratio = Some(ratio);
            if ratio < 1.03 {
                out.reasons.push("Median paired gain is below 1.03".into());
            }
        }
        let b: Vec<_> = self.rows.iter().filter(|r| !r.candidate).collect();
        let c: Vec<_> = self.rows.iter().filter(|r| r.candidate).collect();
        let variation = cv(&b.iter().map(|r| r.seconds).collect::<Vec<_>>())
            .max(cv(&c.iter().map(|r| r.seconds).collect::<Vec<_>>()));
        out.sample_cv = Some(variation);
        if b.len() == 3 && variation > 0.10 {
            out.needs_five_pairs = true;
            out.reasons
                .push("Completion sample CV exceeds 10%; explicitly collect five new pairs".into());
        }
        if b.len() == 5 && variation > 0.15 {
            out.reasons.push("Completion sample CV exceeds 15%".into());
        }
        for (name, metric) in [("response", 0), ("event-loop p95", 1), ("first result", 2)] {
            if metric == 2 && self.scene != MeasuredSceneDto::Live {
                continue;
            }
            let get = |r: &&Row| match metric {
                0 => r.response,
                1 => r.event_loop,
                _ => r.first.unwrap(),
            };
            let bv: Vec<_> = b.iter().map(get).collect();
            let cv: Vec<_> = c.iter().map(get).collect();
            if median(cv.clone()) > median(bv.clone()) * 1.10 + 0.005 {
                out.reasons.push(format!("{name} median regression"));
            }
            if maximum(&cv) > maximum(&bv) * 1.25 + 0.020 {
                out.reasons.push(format!("{name} maximum regression"));
            }
        }
        for (i, row) in self.rows.iter().enumerate() {
            if row.processes != 1 {
                out.reasons.push(format!(
                    "Row {} did not observe exactly one app-owned engine",
                    i + 1
                ));
            }
            if row.visits.iter().any(|v| *v < self.budget) {
                out.reasons
                    .push(format!("Row {} did not complete every position budget", i + 1));
            }
            if row.memory > self.physical_memory {
                out.reasons
                    .push(format!("Row {} exceeds physical GPU memory", i + 1));
            }
        }
        let bm = b.iter().map(|r| r.memory).fold(0.0, f64::max);
        let cm = c.iter().map(|r| r.memory).fold(0.0, f64::max);
        if cm > bm * 1.15 + 256.0 {
            out.reasons
                .push("Sampled total GPU memory exceeds baseline allowance".into());
        }
        if cm > self.physical_memory * 0.90 {
            out.reasons
                .push("Sampled total GPU memory exceeds 90% physical memory".into());
        }
        out.eligible = out.reasons.is_empty();
        out
    }
}
