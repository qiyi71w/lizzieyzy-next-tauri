//! Opt-in real-process acceptance. Only test builds include the wire recorder.
use super::*;
use app_model::*;
use serde_json::{json, Value};
use std::{env, path::PathBuf, process::Command, sync::OnceLock};

static TRACE: OnceLock<Sender<Value>> = OnceLock::new();
pub(super) fn record(direction: &str, text: &str) {
    if let Some(trace) = TRACE.get() {
        let _ = trace.send(json!({"direction": direction, "line": text}));
    }
}

struct Evidence {
    path: PathBuf,
    events: Receiver<Value>,
}
impl Drop for Evidence {
    fn drop(&mut self) {
        let events: Vec<_> = self.events.try_iter().collect();
        std::fs::write(&self.path, serde_json::to_vec_pretty(&events).unwrap()).unwrap();
    }
}
fn ready(manager: &ForegroundEngineManager) -> EngineRunDto {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        match manager.snapshot().lifecycle {
            ForegroundEngineLifecycleDto::Ready { run } => return run,
            ForegroundEngineLifecycleDto::Error { failure, .. } => panic!("{failure:?}"),
            _ => assert!(Instant::now() < deadline, "engine readiness timed out"),
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn request(
    run: &EngineRunDto,
    text: &str,
    depth: usize,
    deadline_ms: u32,
    max_visits: Option<u32>,
) -> GameMoveRequest {
    let document = sgf::CurrentSgfDocument::open(text).unwrap();
    let path = NodePath {
        indices: vec![0; depth],
    };
    let position = document.exact_position(&path).unwrap();
    record("position", &serde_json::to_string(position.dto()).unwrap());
    GameMoveRequest {
        identity: GameMoveRequestDto {
            run_id: run.run_id.clone(),
            generation: 71,
            node_path: path,
            budget: ComputeBudgetDto {
                deadline_ms,
                max_visits,
            },
        },
        position,
    }
}
fn check_document_move(
    manager: &ForegroundEngineManager,
    request: GameMoveRequest,
    document: &sgf::CurrentSgfDocument,
) {
    let before = document.serialize().unwrap();
    let result = manager.start_game_move(request.clone()).unwrap().wait().unwrap();
    if let GameMoveDto::Move { vertex } = &result.result {
        request.position.validate_move(vertex).unwrap();
    }
    assert_eq!(
        result.engine_time_mapped,
        request.identity.budget.max_visits.is_none()
    );
    assert_eq!(document.serialize().unwrap(), before);
    record("result", &serde_json::to_string(&result).unwrap());
}

#[test]
#[ignore = "requires explicit real GNU Go/KataGo paths and an isolated evidence directory"]
fn real_exact_move_smoke() {
    assert_eq!(env::var("LIZZIEYZY_REAL_GAME_MOVE").as_deref(), Ok("1"));
    let directory = PathBuf::from(env::var("LIZZIEYZY_MOVE_EVIDENCE").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let (tx, events) = mpsc::channel();
    TRACE.set(tx).unwrap();
    let _evidence = Evidence {
        path: directory.join("move-transcript.json"),
        events,
    };
    let catalog = Arc::new(crate::InMemoryEngineProfileCatalog::new());
    for kata in [false, true] {
        let program = env::var(if kata {
            "LIZZIEYZY_KATAGO_PROGRAM"
        } else {
            "LIZZIEYZY_GNUGO_PROGRAM"
        })
        .unwrap();
        let version = Command::new(&program)
            .arg(if kata { "version" } else { "--version" })
            .output()
            .unwrap();
        assert!(version.status.success());
        record("version", &String::from_utf8_lossy(&version.stdout));
        let profile = EngineProfileDto {
            name: if kata { "real KataGo" } else { "real GNU Go" }.into(),
            program,
            argv: if kata {
                vec![]
            } else {
                [
                    "--mode",
                    "gtp",
                    "--chinese-rules",
                    "--positional-superko",
                    "--forbid-suicide",
                    "--level",
                    "1",
                    "--seed",
                    "1",
                ]
                .map(String::from)
                .into()
            },
            working_dir: Some(directory.to_string_lossy().into()),
            adapter: if kata {
                EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                    model_path: Some(env::var("LIZZIEYZY_KATAGO_MODEL").unwrap()),
                    config_path: Some(env::var("LIZZIEYZY_KATAGO_CONFIG").unwrap()),
                    max_visits: 16,
                })
            } else {
                EngineAdapterSettings::GenericGtp(GenericGtpSettings {})
            },
        };
        catalog.upsert(SavedEngineProfile {
            profile_id: "real".into(),
            profile,
        });
        let manager = ForegroundEngineManager::new(
            catalog.clone(),
            ForegroundEngineConfig {
                readiness_timeout: Duration::from_secs(120),
                stop_drain_timeout: Duration::from_secs(5),
                ..ForegroundEngineConfig::default()
            },
        );
        manager.start("real").unwrap();
        let run = ready(&manager);
        let pid = manager.lock().live.as_ref().unwrap().process_id;
        record("ready", &serde_json::to_string(&run).unwrap());
        record("pid", &pid.to_string());
        let sgf = "(;SZ[9]RU[Chinese-KGS]KM[6.5]C[personal root];B[dd];W[];B[fg]C[personal move])";
        let document = sgf::CurrentSgfDocument::open(sgf).unwrap();
        check_document_move(
            &manager,
            request(&run, sgf, 3, 30000, kata.then_some(16)),
            &document,
        );
        let white = "(;SZ[9]RU[Chinese-KGS]KM[6.5]PL[W])";
        check_document_move(
            &manager,
            request(&run, white, 0, 30000, kata.then_some(16)),
            &sgf::CurrentSgfDocument::open(white).unwrap(),
        );
        if kata {
            let handicap = "(;SZ[9]RU[Chinese-KGS]KM[0.5]HA[2]AB[cc][gg]PL[W])";
            check_document_move(
                &manager,
                request(&run, handicap, 0, 30000, Some(16)),
                &sgf::CurrentSgfDocument::open(handicap).unwrap(),
            );
            let timed = manager
                .start_game_move(request(&run, sgf, 3, 100, Some(1_000_000)))
                .unwrap();
            assert_eq!(timed.wait().unwrap_err().kind, EngineFailureKind::Timeout);
            assert!(matches!(
                manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Ready { .. }
            ));
            record("timeout", "target-final confirmed; Ready");
            let active = manager
                .start_game_move(request(&run, sgf, 3, 30000, Some(1_000_000)))
                .unwrap();
            thread::sleep(Duration::from_millis(100));
            manager
                .cancel_game_move(&active.identity.run_id, &active.identity.job_id)
                .unwrap();
            assert_eq!(active.wait().unwrap_err().kind, EngineFailureKind::Cancellation);
            assert!(matches!(
                manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::Ready { .. }
            ));
            record("cancel", "target-final confirmed; Ready");
        }
        manager.stop().unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if matches!(
                manager.snapshot().lifecycle,
                ForegroundEngineLifecycleDto::NoEngine { .. }
            ) {
                break;
            }
            assert!(Instant::now() < deadline, "Stop did not reap process");
            thread::sleep(Duration::from_millis(20));
        }
        assert!(manager.lock().live.is_none());
        record("exit", &format!("pid={pid} reaped; NoEngine; document unchanged"));
        assert_eq!(
            document.serialize().unwrap(),
            sgf::CurrentSgfDocument::open(sgf).unwrap().serialize().unwrap()
        );
        manager.teardown().unwrap();
    }
}
