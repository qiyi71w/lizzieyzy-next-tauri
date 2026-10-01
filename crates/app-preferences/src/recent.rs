const MAX_RECENT_PATHS: usize = 5;

/// Prepend `opened`, remove duplicates by string equality or same OS file identity,
/// preserve remaining order, and cap at 5 paths.
pub fn recent_game_paths(existing: &[String], opened: &str) -> Vec<String> {
    let mut result = Vec::with_capacity(existing.len().saturating_add(1).min(MAX_RECENT_PATHS));
    result.push(opened.to_string());

    for path in existing {
        if result.len() >= MAX_RECENT_PATHS {
            break;
        }
        if result.iter().any(|kept| is_same_file_or_equal(kept, path)) {
            continue;
        }
        result.push(path.clone());
    }

    result
}

fn is_same_file_or_equal(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    same_file::is_same_file(a, b).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(name: &str) -> Self {
            let unique = format!(
                "test-{}-{}-{}",
                name,
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
            );
            let path = std::env::temp_dir().join("lizzieyzy-recent-test").join(unique);
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn recent_sixth_eviction_and_repeat_move_front() {
        let existing = vec![
            "/games/game1.sgf".to_string(),
            "/games/game2.sgf".to_string(),
            "/games/game3.sgf".to_string(),
            "/games/game4.sgf".to_string(),
            "/games/game5.sgf".to_string(),
        ];

        // Sixth eviction: opening a 6th distinct file evicts the oldest (5th) entry.
        let with_sixth = recent_game_paths(&existing, "/games/game6.sgf");
        assert_eq!(
            with_sixth,
            vec![
                "/games/game6.sgf",
                "/games/game1.sgf",
                "/games/game2.sgf",
                "/games/game3.sgf",
                "/games/game4.sgf",
            ]
        );

        // Repeat move front: opening an already present entry moves it to index 0
        // and preserves the relative order of all other entries without exceeding cap 5.
        let repeat_move = recent_game_paths(&existing, "/games/game3.sgf");
        assert_eq!(
            repeat_move,
            vec![
                "/games/game3.sgf",
                "/games/game1.sgf",
                "/games/game2.sgf",
                "/games/game4.sgf",
                "/games/game5.sgf",
            ]
        );

        // Repeat opening the front item keeps it at the front without duplicate.
        let repeat_front = recent_game_paths(&existing, "/games/game1.sgf");
        assert_eq!(repeat_front, existing);

        // Repeat opening the tail item moves it to the front and keeps remaining 4 in order.
        let repeat_tail = recent_game_paths(&existing, "/games/game5.sgf");
        assert_eq!(
            repeat_tail,
            vec![
                "/games/game5.sgf",
                "/games/game1.sgf",
                "/games/game2.sgf",
                "/games/game3.sgf",
                "/games/game4.sgf",
            ]
        );
    }

    #[test]
    fn recent_same_name_independent_dirs_preserved() {
        let fixture = TempDirGuard::new("same-name");
        let dir_a = fixture.path().join("tournament_a");
        let dir_b = fixture.path().join("tournament_b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let file_a = dir_a.join("round1.sgf");
        let file_b = dir_b.join("round1.sgf");
        fs::write(&file_a, "(;GM[1]FF[4]EV[Tournament A])").unwrap();
        fs::write(&file_b, "(;GM[1]FF[4]EV[Tournament B])").unwrap();

        let path_a = file_a.to_str().unwrap().to_string();
        let path_b = file_b.to_str().unwrap().to_string();

        let existing = vec![path_a.clone()];
        let updated = recent_game_paths(&existing, &path_b);

        // Files with the same filename in distinct directories have different OS file identities
        // and must both be retained in the recent history.
        assert_eq!(updated, vec![path_b, path_a]);
    }

    #[test]
    fn recent_hardlink_alias_dedup() {
        let fixture = TempDirGuard::new("hardlink");
        let original_file = fixture.path().join("original.sgf");
        let hardlink_file = fixture.path().join("alias_link.sgf");
        let other_file = fixture.path().join("other.sgf");

        fs::write(&original_file, "(;GM[1]FF[4]EV[Original])").unwrap();
        fs::write(&other_file, "(;GM[1]FF[4]EV[Other])").unwrap();
        fs::hard_link(&original_file, &hardlink_file).unwrap();

        let original_str = original_file.to_str().unwrap().to_string();
        let hardlink_str = hardlink_file.to_str().unwrap().to_string();
        let other_str = other_file.to_str().unwrap().to_string();

        // When opening a hardlink to a file already in the list, the older entry
        // is identified as the same OS file and deduplicated.
        let existing = vec![other_str.clone(), original_str.clone()];
        let updated = recent_game_paths(&existing, &hardlink_str);
        assert_eq!(updated, vec![hardlink_str.clone(), other_str.clone()]);

        // Symmetrically, opening the original file when the hardlink is in the list
        // dedups the hardlink.
        let existing_link = vec![other_str.clone(), hardlink_str];
        let updated_orig = recent_game_paths(&existing_link, &original_str);
        assert_eq!(updated_orig, vec![original_str, other_str]);
    }

    #[test]
    fn recent_vanished_path_retained() {
        let fixture = TempDirGuard::new("vanished");
        let live_file = fixture.path().join("live.sgf");
        let vanished_file = fixture.path().join("vanished.sgf");

        fs::write(&live_file, "(;GM[1]FF[4]EV[Live])").unwrap();
        fs::write(&vanished_file, "(;GM[1]FF[4]EV[To disappear])").unwrap();

        let live_str = live_file.to_str().unwrap().to_string();
        let vanished_str = vanished_file.to_str().unwrap().to_string();

        // Delete the vanished file from the filesystem.
        fs::remove_file(&vanished_file).unwrap();
        assert!(!vanished_file.exists());

        // Opening another file must retain the vanished path from historical records.
        // Comparison errors for missing files treat them as not equal (unless strings match).
        let existing = vec![vanished_str.clone()];
        let updated = recent_game_paths(&existing, &live_str);
        assert_eq!(updated, vec![live_str, vanished_str.clone()]);

        // Opening a vanished file itself must record it at the front without canonicalization failure.
        let opened_ghost = recent_game_paths(std::slice::from_ref(&vanished_str), &vanished_str);
        assert_eq!(opened_ghost, vec![vanished_str]);
    }
}
