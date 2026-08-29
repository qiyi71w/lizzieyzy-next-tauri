use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveAsDialogOutcome {
    Cancelled,
    Chosen(String),
    Denied { requested: String },
    Redirected { requested: String, redirected: String },
}

pub fn persist_save_as<S, T>(outcome: SaveAsDialogOutcome, save_to_path: S) -> Result<Option<T>, String>
where
    S: FnOnce(String) -> Result<T, String>,
{
    match outcome {
        SaveAsDialogOutcome::Cancelled => Ok(None),
        SaveAsDialogOutcome::Chosen(path) => save_to_path(path).map(Some),
        SaveAsDialogOutcome::Denied { requested } => Err(write_failure(&requested)),
        SaveAsDialogOutcome::Redirected {
            requested,
            redirected: _,
        } => Err(write_failure(&requested)),
    }
}

fn write_failure(requested: &str) -> String {
    format!("failed to write SGF file {requested}: access denied")
}

pub fn classify_save_as_session(
    requested_unwritable: Option<String>,
    returned: Option<String>,
) -> SaveAsDialogOutcome {
    match (requested_unwritable, returned) {
        (_, None) => SaveAsDialogOutcome::Cancelled,
        (None, Some(path)) => SaveAsDialogOutcome::Chosen(path),
        (Some(requested), Some(returned)) if same_save_path(&requested, &returned) => {
            SaveAsDialogOutcome::Chosen(returned)
        }
        (Some(requested), Some(returned)) if looks_like_user_folder_redirect(&requested, &returned) => {
            SaveAsDialogOutcome::Redirected {
                requested,
                redirected: returned,
            }
        }
        (Some(_requested), Some(returned)) => SaveAsDialogOutcome::Chosen(returned),
    }
}

fn same_save_path(left: &str, right: &str) -> bool {
    path_key(Path::new(left.trim())) == path_key(Path::new(right.trim()))
}

fn looks_like_user_folder_redirect(requested: &str, returned: &str) -> bool {
    let requested_parts = win_path_segments(requested);
    let returned_parts = win_path_segments(returned);
    match (requested_parts.last(), returned_parts.split_last()) {
        (Some(requested_name), Some((returned_name, parent))) if requested_name == returned_name => {
            is_windows_shell_fallback_parent(parent)
        }
        _ => false,
    }
}

fn win_path_segments(path: &str) -> Vec<String> {
    path.trim()
        .split(['\\', '/'])
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .map(|part| part.to_ascii_lowercase())
        .collect()
}

fn is_windows_shell_fallback_parent(parent: &[String]) -> bool {
    let Some(users) = parent.iter().position(|part| part == "users") else {
        return false;
    };
    match &parent[users + 1..] {
        [_user] => true,
        [_user, folder] if folder == "documents" || folder == "desktop" => true,
        _ => false,
    }
}

fn path_key(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_ascii_lowercase()),
            Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().to_ascii_lowercase()),
            Component::RootDir => Some(String::from("/")),
            Component::CurDir | Component::ParentDir => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn save_as_dialog_redirect_leaves_path_and_dirty_unchanged() {
        let wrote = Cell::new(false);
        let mut dirty = true;
        let mut native_path = Some("/tmp/branching.sgf".to_string());
        let redirected = "/home/admin/denied.sgf";
        let requested = "/denied-acl/denied.sgf";

        let error = persist_save_as(
            SaveAsDialogOutcome::Redirected {
                requested: requested.to_string(),
                redirected: redirected.to_string(),
            },
            |path| {
                wrote.set(true);
                dirty = false;
                native_path = Some(path);
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.contains("failed to write"), "{error}");
        assert!(error.contains(requested), "{error}");
        assert!(!wrote.get(), "redirected path must not be written");
        assert!(dirty);
        assert_eq!(native_path.as_deref(), Some("/tmp/branching.sgf"));
    }

    #[test]
    fn save_as_dialog_denied_leaves_path_and_dirty_unchanged() {
        let wrote = Cell::new(false);
        let mut dirty = true;
        let mut native_path = Some("/tmp/branching.sgf".to_string());
        let requested = "/denied-acl/denied.sgf";

        let error = persist_save_as(
            SaveAsDialogOutcome::Denied {
                requested: requested.to_string(),
            },
            |path| {
                wrote.set(true);
                dirty = false;
                native_path = Some(path);
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.contains("failed to write"), "{error}");
        assert!(error.contains(requested), "{error}");
        assert!(!wrote.get());
        assert!(dirty);
        assert_eq!(native_path.as_deref(), Some("/tmp/branching.sgf"));
    }

    #[test]
    fn save_as_dialog_cancel_leaves_path_and_dirty_unchanged() {
        let wrote = Cell::new(false);
        let mut dirty = true;
        let mut native_path = Some("/tmp/branching.sgf".to_string());

        let result = persist_save_as(SaveAsDialogOutcome::Cancelled, |path| {
            wrote.set(true);
            dirty = false;
            native_path = Some(path);
            Ok(())
        })
        .unwrap();

        assert_eq!(result, None);
        assert!(!wrote.get());
        assert!(dirty);
        assert_eq!(native_path.as_deref(), Some("/tmp/branching.sgf"));
    }

    #[test]
    fn save_as_dialog_chosen_writes_and_clears_dirty() {
        let wrote = Cell::new(false);
        let chosen = "/tmp/allowed.sgf";

        let result = persist_save_as(SaveAsDialogOutcome::Chosen(chosen.to_string()), |path| {
            wrote.set(true);
            assert_eq!(path, chosen);
            Ok("saved")
        })
        .unwrap();

        assert_eq!(result, Some("saved"));
        assert!(wrote.get());
    }

    #[test]
    fn classify_save_as_session_represents_windows_acl_redirect() {
        let outcome = classify_save_as_session(
            Some(String::from(
                r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf",
            )),
            Some(String::from(r"C:\Users\admin\denied.sgf")),
        );

        assert_eq!(
            outcome,
            SaveAsDialogOutcome::Redirected {
                requested: String::from(r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"),
                redirected: String::from(r"C:\Users\admin\denied.sgf"),
            }
        );
    }

    #[test]
    fn classify_save_as_session_keeps_cancel_and_allowed_choice() {
        assert_eq!(
            classify_save_as_session(None, None),
            SaveAsDialogOutcome::Cancelled
        );
        assert_eq!(
            classify_save_as_session(Some(String::from(r"D:\denied\denied.sgf")), None),
            SaveAsDialogOutcome::Cancelled
        );
        assert_eq!(
            classify_save_as_session(None, Some(String::from("/tmp/allowed.sgf"))),
            SaveAsDialogOutcome::Chosen(String::from("/tmp/allowed.sgf"))
        );
    }

    #[test]
    fn classify_save_as_session_keeps_later_allowed_choice_after_denied_path() {
        let allowed = String::from(r"D:\dev\weiqi\tmp\editable-workspace-branching.sgf");
        assert_eq!(
            classify_save_as_session(
                Some(String::from(
                    r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"
                )),
                Some(allowed.clone()),
            ),
            SaveAsDialogOutcome::Chosen(allowed)
        );
        assert_eq!(
            classify_save_as_session(
                Some(String::from(
                    r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"
                )),
                Some(String::from(r"C:\Users\admin\Documents\denied.sgf")),
            ),
            SaveAsDialogOutcome::Redirected {
                requested: String::from(r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"),
                redirected: String::from(r"C:\Users\admin\Documents\denied.sgf"),
            }
        );
        assert_eq!(
            classify_save_as_session(
                Some(String::from(
                    r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"
                )),
                Some(String::from(r"C:\Users\admin\Desktop\denied.sgf")),
            ),
            SaveAsDialogOutcome::Redirected {
                requested: String::from(r"D:\dev\weiqi\tmp\editable-sgf-08-acl-deny\denied.sgf"),
                redirected: String::from(r"C:\Users\admin\Desktop\denied.sgf"),
            }
        );
    }
}
