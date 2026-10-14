//! Application observations supplied only after current-game adoption and lifecycle fences.
use super::{
    fingerprint::{self, GpuSample},
    parse,
};
use app_model::ApplicationMeasurementRequestDto;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub fn validate_request(request: &ApplicationMeasurementRequestDto) -> Result<(), String> {
    if !request.replace_current_document {
        return Err("Explicit fixture document replacement consent is required".into());
    }
    if !matches!(request.pairs, 3 | 5) || request.budget == 0 || request.budget > i32::MAX as u32 {
        return Err("Measurement requires 3 or 5 pairs and a positive i32 visit budget".into());
    }
    if request.fixture_sgf.is_empty() || request.fixture_sgf.len() > 1024 * 1024 {
        return Err("Measurement fixture must be nonempty and at most 1 MiB".into());
    }
    super::parameters(
        &serde_json::to_value(&request.candidate_parameters).map_err(|e| e.to_string())?,
        request.scene,
    )?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct MemoryMeasurements {
    pub identity: GpuSample,
    pub max_memory_mib: f64,
    pub max_engine_processes: usize,
    pub samples: Vec<(f64, f64, usize)>,
}
/// A sampler borrows the existing manager for observations; it never owns an engine child.
pub struct MemorySampler {
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<Result<MemoryMeasurements, String>>>,
    started: Instant,
    manager: crate::ForegroundEngineManager,
}
impl MemorySampler {
    pub fn start(manager: &crate::ForegroundEngineManager) -> Result<Self, String> {
        let started = Instant::now();
        let identity = fingerprint::sample_gpu()?;
        let initial = (
            started.elapsed().as_secs_f64(),
            identity.used_mib,
            manager.measured_process_count(),
        );
        let stop = Arc::new(AtomicBool::new(false));
        let cancelled = stop.clone();
        let observed = manager.clone();
        let worker = thread::spawn(move || {
            let mut measured = MemoryMeasurements {
                max_memory_mib: identity.used_mib,
                max_engine_processes: initial.2,
                identity,
                samples: vec![initial],
            };
            while !cancelled.load(Ordering::Acquire) {
                let sample = fingerprint::sample_gpu()?;
                if sample.uuid != measured.identity.uuid
                    || sample.driver != measured.identity.driver
                    || sample.name != measured.identity.name
                    || sample.physical_mib != measured.identity.physical_mib
                {
                    return Err("GPU identity changed during application measurement".into());
                }
                let processes = observed.measured_process_count();
                measured.max_memory_mib = measured.max_memory_mib.max(sample.used_mib);
                measured.max_engine_processes = measured.max_engine_processes.max(processes);
                measured
                    .samples
                    .push((started.elapsed().as_secs_f64(), sample.used_mib, processes));
                for _ in 0..20 {
                    if cancelled.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
            Ok(measured)
        });
        Ok(Self {
            stop,
            worker: Some(worker),
            started,
            manager: manager.clone(),
        })
    }
    pub fn finish(mut self) -> Result<MemoryMeasurements, String> {
        self.stop.store(true, Ordering::Release);
        let mut measured = self
            .worker
            .take()
            .expect("sampler worker")
            .join()
            .map_err(|_| "GPU sampler worker failed".to_string())??;
        let last = fingerprint::sample_gpu()?;
        if last.uuid != measured.identity.uuid
            || last.driver != measured.identity.driver
            || last.name != measured.identity.name
            || last.physical_mib != measured.identity.physical_mib
        {
            return Err("GPU identity changed at actual reap".into());
        }
        let processes = self.manager.measured_process_count();
        if processes != 0 {
            return Err("GPU final sample preceded actual engine reap".into());
        }
        measured.max_memory_mib = measured.max_memory_mib.max(last.used_mib);
        measured
            .samples
            .push((self.started.elapsed().as_secs_f64(), last.used_mib, processes));
        Ok(measured)
    }
}
impl Drop for MemorySampler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub fn assemble(
    request: &ApplicationMeasurementRequestDto,
    fingerprint: Value,
    baseline: std::collections::BTreeMap<String, u32>,
    positions: usize,
    runs: Vec<Value>,
    cold: Vec<Value>,
) -> Result<app_model::ApplicationMeasurementResultDto, String> {
    let report = json!({"schemaVersion":1,"scene":request.scene,"measurementMode":"application","metricScope":"totalGpu","fingerprint":fingerprint,"fixtureSha256":format!("{:x}",Sha256::digest(request.fixture_sgf.as_bytes())),"budget":request.budget,"positions":positions,"baselineParameters":baseline,"candidateParameters":request.candidate_parameters,"runs":runs});
    parse(&serde_json::to_vec(&report).map_err(|e| e.to_string())?)?;
    Ok(app_model::ApplicationMeasurementResultDto {
        report,
        evidence: json!({"coldAndSampling":cold,"ramStatus":"experimental-unmeasured","gpuMetric":"sampled lifecycle total GPU, not continuous peak or per-process"}),
    })
}
