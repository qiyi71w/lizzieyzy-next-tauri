//! Saved-entry recommendations are qualified observations, never implicit configuration edits.
use crate::{
    diagnostics::AttemptCapture,
    katago_config::{Configuration, PolicyOverride},
    resources::ResourceSnapshot,
    CommandSpec, SavedEngineProfile,
};
use app_model::*;
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(crate) const SOURCE_REVISION: &str =
    "katago:47aadc08518b3e121f22539796c911002f699584:computeEloEffect:recommended:v1";

pub(crate) fn input_revision(profile: &EngineProfileDto) -> String {
    digest(&serde_json::to_vec(profile).expect("serializable profile"))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn command_revision(spec: &CommandSpec) -> Result<String, String> {
    let cwd = match &spec.working_dir {
        Some(path) => path.clone(),
        None => std::env::current_dir()
            .map_err(|_| "policy_environment_unknown")?
            .to_string_lossy()
            .into_owned(),
    };
    Ok(digest(
        &serde_json::to_vec(&(&spec.program, &spec.args, cwd, &spec.env)).expect("serializable command"),
    ))
}

/// These are observed host/process facts, compared again at Save and launch. No remote probe.
pub(crate) fn environment() -> Result<String, String> {
    let parallelism = std::thread::available_parallelism()
        .map_err(|_| "policy_environment_unknown")?
        .get();
    let host = if cfg!(target_os = "linux") {
        std::fs::read_to_string("/etc/machine-id").map_err(|_| "policy_environment_unknown")?
    } else if cfg!(target_os = "windows") {
        format!(
            "{}:{}",
            std::env::var("COMPUTERNAME").map_err(|_| "policy_environment_unknown")?,
            std::env::var("PROCESSOR_IDENTIFIER").map_err(|_| "policy_environment_unknown")?
        )
    } else {
        return Err("policy_environment_unknown".into());
    };
    if host.trim().is_empty() {
        return Err("policy_environment_unknown".into());
    }
    let mut vars = std::env::vars_os()
        .map(|(k, v)| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned()))
        .collect::<Vec<_>>();
    vars.sort();
    if vars.iter().any(|(k, v)| {
        matches!(k.as_str(), "LD_PRELOAD" | "LD_AUDIT" | "DYLD_INSERT_LIBRARIES") && !v.is_empty()
    }) {
        return Err("policy_environment_unknown".into());
    }
    Ok(digest(
        &serde_json::to_vec(&(
            std::env::consts::OS,
            std::env::consts::ARCH,
            host,
            parallelism,
            vars,
        ))
        .expect("serializable environment"),
    ))
}

/// Reuse the release owner's pinned native CPU bytes. A filename/version subprocess is not proof.
pub(crate) fn direct_local(resources: &QualifiedLocalResourceDto) -> bool {
    #[cfg(test)]
    if resources.resources.iter().any(|r| {
        r.component == "executable"
            && r.sha256 == digest(include_bytes!("../tests/fixtures/evaluation_process.py"))
    }) {
        return true;
    }
    let catalog = crate::managed::catalog_snapshot();
    resources
        .resources
        .iter()
        .find(|r| r.component == "executable")
        .is_some_and(|exe| {
            catalog.targets.iter().any(|target| {
                target.hardware_qualification == "host_cpu" && target.executable_sha256 == exe.sha256
            })
        })
}

pub(crate) fn capture(
    saved: &SavedEngineProfile,
    spec: &CommandSpec,
    managed_root: Option<&Path>,
) -> Result<QualifiedLocalResourceDto, String> {
    let run = EngineRunDto {
        run_id: "policy-qualification".into(),
        profile_id: saved.profile_id.clone(),
        adapter_kind: saved.profile.adapter_kind(),
        profile_snapshot: saved.profile.clone(),
        thread_policy_snapshot: saved.thread_policy.clone(),
        capability_snapshot: None,
        qualified_resource: None,
    };
    let diagnostics = AttemptCapture::new(&run);
    let deadline = Instant::now() + Duration::from_secs(30);
    let configuration = Configuration::from_command(spec, deadline).map_err(|_| "policy_invalid_config")?;
    configuration.benchmark_compatible()?;
    configuration
        .policy_override(saved.profile.adapter_kind(), &diagnostics)
        .map_err(|_| "policy_invalid_config")?;
    let resources =
        ResourceSnapshot::capture_evaluation(&run, spec, deadline, diagnostics, managed_root, &|| false)
            .map_err(|(_, message)| message)?
            .qualified(None, None);
    Ok(resources)
}

pub(crate) fn source_threads(row: &str) -> Option<u32> {
    let (threads, elo) = row.strip_prefix("numSearchThreads =")?.split_once(':')?;
    let threads = threads
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| (1..=4096).contains(n))?;
    let elo = elo.trim().strip_suffix(" (recommended)")?;
    if elo != "(baseline)" {
        elo.strip_suffix(" Elo")?.trim().parse::<i32>().ok()?;
    }
    Some(threads)
}

pub(crate) fn validity(
    saved: &SavedEngineProfile,
    recommendation: Option<&ThreadRecommendationDto>,
    spec: &CommandSpec,
    managed_root: Option<&Path>,
) -> (RecommendationValidityDto, String) {
    use RecommendationValidityDto::*;
    let Some(rec) = recommendation else {
        return (None, "policy_none".into());
    };
    if saved.profile.adapter_kind() == EngineBackend::GenericGtp {
        return (Invalid, "policy_invalid".into());
    }
    if !(1..=4096).contains(&rec.threads)
        || rec.source_revision != SOURCE_REVISION
        || rec.seconds_per_move != "5"
        || source_threads(&rec.source_row) != Some(rec.threads)
        || !direct_local(&rec.qualified_resource)
    {
        return (Invalid, "policy_invalid".into());
    }
    if rec.target_id != saved.profile_id || rec.input_revision != input_revision(&saved.profile) {
        return (Changed, "policy_input_changed".into());
    }
    let (Ok(command), Ok(environment)) = (command_revision(spec), environment()) else {
        return (Unknown, "policy_environment_unknown".into());
    };
    if command != rec.command_revision || environment != rec.environment {
        return (Changed, "policy_environment_changed".into());
    }
    match capture(saved, spec, managed_root) {
        Ok(resources)
            if resources.resources.len() == rec.qualified_resource.resources.len()
                && resources
                    .resources
                    .iter()
                    .zip(&rec.qualified_resource.resources)
                    .all(|(actual, expected)| crate::resources::same_content(actual, expected)) =>
        {
            (Valid, "policy_valid".into())
        }
        Ok(_) => (Changed, "policy_resources_changed".into()),
        Err(reason) if reason == "policy_invalid_config" || reason == "policy_environment_mismatch" => {
            (Invalid, reason)
        }
        Err(reason) => (Unknown, reason),
    }
}

/// Returns whether launch must retain the recommendation's resource receipt.
pub(crate) fn apply(
    run: &EngineRunDto,
    spec: &mut CommandSpec,
    managed_root: Option<&Path>,
) -> Result<bool, String> {
    if run.thread_policy_snapshot.strategy == ThreadStrategyDto::Cfg {
        return Ok(false);
    }
    let capture = AttemptCapture::new(run);
    let configuration = Configuration::from_command(spec, Instant::now() + Duration::from_secs(30))
        .map_err(|e| e.diagnostic(&capture))?;
    let explicit = configuration.policy_override(run.adapter_kind, &capture)?;
    if matches!(explicit, PolicyOverride::Legal) {
        return Ok(false);
    }
    let saved = SavedEngineProfile {
        profile_id: run.profile_id.clone(),
        profile: run.profile_snapshot.clone(),
        thread_policy: run.thread_policy_snapshot.clone(),
    };
    let (valid, reason) = validity(
        &saved,
        saved.thread_policy.recommendation.as_ref(),
        spec,
        managed_root,
    );
    if valid != RecommendationValidityDto::Valid {
        return Err(reason);
    }
    // Deletion is an intentional final CLI operation, not absence. Never undo it.
    if matches!(explicit, PolicyOverride::Deleted) {
        return Ok(true);
    }
    let recommendation = saved
        .thread_policy
        .recommendation
        .as_ref()
        .expect("valid recommendation");
    let key = if run.adapter_kind == EngineBackend::KataGoAnalysis
        && configuration
            .thread_sources(run.adapter_kind, &capture)
            .entries
            .iter()
            .any(|e| e.key == "numSearchThreadsPerAnalysisThread")
    {
        "numSearchThreadsPerAnalysisThread"
    } else {
        "numSearchThreads"
    };
    // Merge into the final override argument: upstream treats repeated CLI flags as one override list.
    if let Some(index) = spec.args.iter().rposition(|arg| arg == "-override-config") {
        let value = spec.args.get_mut(index + 1).ok_or("policy_invalid_override")?;
        if !value.is_empty() {
            value.push(',');
        }
        value.push_str(&format!("{key}={}", recommendation.threads));
    } else {
        spec.args.extend([
            "-override-config".into(),
            format!("{key}={}", recommendation.threads),
        ]);
    }
    Ok(true)
}

pub(crate) fn verify_launch_resources(
    run: &EngineRunDto,
    resources: &ResourceSnapshot,
) -> Result<(), String> {
    let rec = run
        .thread_policy_snapshot
        .recommendation
        .as_ref()
        .ok_or("policy_none")?;
    if !resources.matches_resources(&rec.qualified_resource.resources) {
        return Err("policy_resources_changed".into());
    }
    Ok(())
}
