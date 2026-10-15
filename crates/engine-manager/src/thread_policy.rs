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
const ENVIRONMENT_REVISION: &str = "katago-eigen-env:v2:";

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

/// Host facts and the pinned CPU engine's effective environment, rechecked at Save and launch.
pub(crate) fn environment(spec: &CommandSpec) -> Result<String, String> {
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
    let vars = engine_environment(std::env::vars_os(), &spec.env, cfg!(windows))?;
    Ok(format!(
        "{ENVIRONMENT_REVISION}{}",
        digest(
            &serde_json::to_vec(&(
                std::env::consts::OS,
                std::env::consts::ARCH,
                host,
                parallelism,
                vars,
            ))
            .expect("serializable environment")
        )
    ))
}

fn engine_environment(
    inherited: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
    overrides: &[(String, String)],
    windows: bool,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut vars = std::collections::BTreeMap::new();
    for (key, value) in inherited
        .into_iter()
        .chain(overrides.iter().map(|(key, value)| (key.into(), value.into())))
    {
        let key = key.to_str().ok_or("policy_environment_unknown")?;
        let matches_key = |expected: &str| {
            if windows {
                key.eq_ignore_ascii_case(expected)
            } else {
                key == expected
            }
        };
        // Parent injection also makes this process's qualification unconfirmable, even if
        // a child override removes it. Never treat injected libraries as ordinary inputs.
        if ["LD_PRELOAD", "LD_AUDIT", "DYLD_INSERT_LIBRARIES"]
            .into_iter()
            .any(matches_key)
            && !value.is_empty()
        {
            return Err("policy_environment_unknown".into());
        }
        // Pinned KataGo HomeData reads HOME on Linux; Windows uses the executable directory.
        // Its Linux libc/loader consume LD_*, GLIBC_TUNABLES and MALLOC_*; Windows DLL search
        // consumes PATH. Eigen has no OpenMP/OpenBLAS environment controls in this build.
        let relevant = if windows {
            matches_key("PATH")
        } else {
            key == "HOME" || key.starts_with("LD_") || key == "GLIBC_TUNABLES" || key.starts_with("MALLOC_")
        };
        if relevant {
            vars.insert(
                if windows { "PATH".into() } else { key.into() },
                value.to_str().ok_or("policy_environment_unknown")?.into(),
            );
        }
    }
    Ok(vars)
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
        measured_scenes_snapshot: Default::default(),
        measured_scene: None,
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
    if !rec.environment.starts_with(ENVIRONMENT_REVISION) {
        return (Changed, "policy_environment_changed".into());
    }
    let (Ok(command), Ok(environment)) = (command_revision(spec), environment(spec)) else {
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
        measured_scenes: Default::default(),
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

#[cfg(test)]
mod environment_tests {
    use super::*;

    fn projected(
        values: &[(&str, &str)],
        overrides: &[(&str, &str)],
        windows: bool,
    ) -> Result<std::collections::BTreeMap<String, String>, String> {
        engine_environment(
            values.iter().map(|(key, value)| ((*key).into(), (*value).into())),
            &overrides
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect::<Vec<_>>(),
            windows,
        )
    }

    #[test]
    fn launcher_metadata_does_not_change_cpu_qualification() {
        let baseline = projected(&[("HOME", "/home/player")], &[], false).unwrap();
        assert_eq!(
            baseline,
            projected(
                &[
                    ("LAUNCHER_SESSION", "new"),
                    ("HOME", "/home/player"),
                    ("TERM", "xterm")
                ],
                &[],
                false
            )
            .unwrap()
        );
        for key in [
            "HOME",
            "LD_LIBRARY_PATH",
            "LD_BIND_NOW",
            "GLIBC_TUNABLES",
            "MALLOC_ARENA_MAX",
        ] {
            assert_ne!(
                projected(&[(key, "before")], &[], false),
                projected(&[(key, "after")], &[], false),
                "{key}"
            );
        }
    }

    #[test]
    fn effective_overrides_and_windows_case_follow_child_environment() {
        assert_eq!(
            projected(&[("HOME", "before")], &[("HOME", "after")], false),
            projected(&[("HOME", "after")], &[], false)
        );
        assert_eq!(
            projected(&[("Path", "before")], &[("PATH", "after")], true),
            projected(&[("path", "after")], &[], true)
        );
        assert_ne!(
            projected(&[("Path", "before")], &[], true),
            projected(&[("PATH", "after")], &[], true)
        );
        assert_eq!(
            projected(&[("HOME", "unused"), ("USERPROFILE", "unused")], &[], true),
            projected(&[], &[], true)
        );
        assert_ne!(projected(&[], &[], false), projected(&[("HOME", "")], &[], false));
    }

    #[test]
    fn injection_is_unknown_in_parent_or_explicit_child_environment() {
        for key in ["LD_PRELOAD", "LD_AUDIT", "DYLD_INSERT_LIBRARIES"] {
            assert_eq!(
                projected(&[(key, "library")], &[(key, "")], false),
                Err("policy_environment_unknown".into())
            );
            assert_eq!(
                projected(&[], &[(key, "library")], false),
                Err("policy_environment_unknown".into())
            );
        }
        assert!(projected(&[("ld_preload", "library")], &[], true).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn relevant_non_unicode_is_unknown_not_a_lossy_identity() {
        use std::os::unix::ffi::OsStringExt;
        let invalid = std::ffi::OsString::from_vec(vec![0xff]);
        assert!(engine_environment([("HOME".into(), invalid.clone())], &[], false).is_err());
        assert!(engine_environment([("LAUNCHER_SESSION".into(), invalid)], &[], false).is_ok());
    }
}
