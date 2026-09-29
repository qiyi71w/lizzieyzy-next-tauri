use super::*;
use engine_manager::{ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog};
use std::{cell::{Cell, RefCell}, collections::VecDeque, sync::Arc};

struct Window {
    actual: Cell<bool>,
    actions: RefCell<VecDeque<(bool, bool, bool)>>,
    unreadable: Cell<bool>,
}
impl Window {
    fn new(actual: bool, actions: Vec<(bool, bool, bool)>) -> Self {
        Self { actual: Cell::new(actual), actions: RefCell::new(actions.into()), unreadable: Cell::new(false) }
    }
}
impl PinWindow for Window {
    fn set_pin(&self, value: bool) -> Result<(), String> {
        let (change, fail, unreadable) = self.actions.borrow_mut().pop_front().unwrap_or((true, false, false));
        if change { self.actual.set(value); }
        self.unreadable.set(unreadable);
        if fail { Err("native setter failed".into()) } else { Ok(()) }
    }
    fn read_pin(&self) -> Result<bool, String> {
        if self.unreadable.get() { Err("native query failed".into()) } else { Ok(self.actual.get()) }
    }
}
struct Fixture {
    directory: std::path::PathBuf,
    path: std::path::PathBuf,
    preferences: PreferencesState,
    manager: ForegroundEngineManager,
    pin: MainWindowPin,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("window-pin-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests());
        let preferences = PreferencesState::default();
        let loaded = preferences.load(&path, &manager).unwrap().preferences;
        preferences.save(&path, &manager, loaded).unwrap();
        Self { directory, path, preferences, manager, pin: MainWindowPin::default() }
    }
    fn apply(&self, window: &Window, value: Option<bool>) -> MainWindowPinStatusDto {
        self.pin.apply(window, &self.preferences, &self.path, value).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) { std::fs::remove_dir_all(&self.directory).unwrap(); }
}

#[test]
fn apply_failure_reads_actual_without_writing_and_retry_restores_durable() {
    for (changes, unreadable, expected) in [(false, false, Some(false)), (true, false, Some(true)), (true, true, None)] {
        let fixture = Fixture::new();
        let original = std::fs::read(&fixture.path).unwrap();
        let window = Window::new(false, vec![(changes, true, unreadable)]);
        let failed = fixture.apply(&window, Some(true));
        assert_eq!(failed.actual, expected);
        assert!(!failed.durable);
        assert!(failed.error.unwrap().contains("native setter failed"));
        assert_eq!(std::fs::read(&fixture.path).unwrap(), original);
        let retry = fixture.apply(&window, None);
        assert_eq!(retry.actual, Some(false));
        assert_eq!(retry.error, None);
        assert_eq!(std::fs::read(&fixture.path).unwrap(), original);
    }
}

#[test]
fn disk_failure_rolls_back_previous_actual_even_when_durable_differs() {
    for (changes, unreadable, expected) in [(false, false, Some(false)), (true, false, Some(true)), (true, true, None)] {
        let fixture = Fixture::new();
        let original = std::fs::read(&fixture.path).unwrap();
        let window = Window::new(true, vec![(true, false, false), (changes, true, unreadable)]);
        let result = fixture.pin.apply(&window, &fixture.preferences, &fixture.directory, Some(false)).unwrap();
        assert!(!result.durable);
        assert_eq!(result.actual, expected);
        assert!(result.error.unwrap().contains("rollback failed"));
        assert_eq!(std::fs::read(&fixture.path).unwrap(), original);
        let retry = fixture.apply(&window, None);
        assert_eq!(retry.actual, Some(false));
        assert_eq!(retry.error, None);
        assert_eq!(std::fs::read(&fixture.path).unwrap(), original);
    }
}

#[test]
fn narrow_pin_commit_survives_stale_full_save_recent_history_and_restart() {
    let fixture = Fixture::new();
    let mut stale = fixture.preferences.load(&fixture.path, &fixture.manager).unwrap().preferences;
    stale.sound_enabled = false;
    let window = Window::new(false, vec![]);
    let pinned = fixture.apply(&window, Some(true));
    assert_eq!(pinned.actual, Some(true));
    assert!(pinned.durable);
    fixture.preferences.update_recent_history(&fixture.path, Some("/games/kept.sgf")).unwrap();
    let saved = fixture.preferences.save(&fixture.path, &fixture.manager, stale).unwrap();
    assert!(saved.main_window_always_on_top);
    assert!(!saved.sound_enabled);
    assert_eq!(saved.recent_game_paths, ["/games/kept.sgf"]);
    let reloaded = PreferencesState::default();
    assert_eq!(reloaded.load(&fixture.path, &fixture.manager).unwrap().preferences, saved);
    let restarted = Window::new(false, vec![]);
    let startup = MainWindowPin::default().apply(&restarted, &reloaded, &fixture.path, None).unwrap();
    assert_eq!(startup.actual, Some(true));
    assert_eq!(startup.error, None);
    assert_eq!(fixture.apply(&window, Some(false)).actual, Some(false));
    assert!(!app_preferences::load_from_path(&fixture.path).unwrap().preferences.main_window_always_on_top);
}
