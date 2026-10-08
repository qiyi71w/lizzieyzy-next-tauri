use crate::CurrentSgfDocument;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Serialize an immutable full document and commit it with a same-directory rename.
/// No failure path truncates, removes, or directly overwrites the destination.
pub fn save_document_atomic(document: &CurrentSgfDocument, path: &Path) -> Result<(), String> {
    write_with(
        path,
        || document.serialize().map_err(|error| error.to_string()),
        |_| Ok(()),
    )
}

/// Atomically replace the final, already-confirmed target with captured bytes.
/// Target normalization and overwrite confirmation belong to the caller.
pub fn write_file_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    write_with(path, || Ok(contents), |_| Ok(()))
}

#[derive(Clone, Copy, Debug)]
enum Stage {
    Prepare,
    Encode,
    Write,
    Replace,
}

fn write_with<T: AsRef<[u8]>>(
    path: &Path,
    encode: impl FnOnce() -> Result<T, String>,
    before: impl Fn(Stage) -> io::Result<()>,
) -> Result<(), String> {
    let commit = || -> Result<(), String> {
        before(Stage::Prepare).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        let existing_permissions = match fs::metadata(path) {
            Ok(metadata) => Some(metadata.permissions()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.to_string()),
        };
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = parent.join(format!(".lizzieyzy-sgf-{}.tmp", uuid::Uuid::new_v4()));
        let mut open_options = OpenOptions::new();
        open_options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            open_options.mode(0o600);
        }
        let mut file = open_options.open(&temporary).map_err(|error| error.to_string())?;
        let temporary = Temporary(temporary);
        let prepared = (|| -> Result<(), String> {
            before(Stage::Encode).map_err(|error| error.to_string())?;
            let contents = encode()?;
            before(Stage::Write).map_err(|error| error.to_string())?;
            file.write_all(contents.as_ref())
                .map_err(|error| error.to_string())?;
            #[cfg(unix)]
            if let Some(permissions) = &existing_permissions {
                file.set_permissions(permissions.clone())
                    .map_err(|error| error.to_string())?;
            }
            file.sync_all().map_err(|error| error.to_string())
        })();
        drop(file);
        prepared?;
        before(Stage::Replace).map_err(|error| error.to_string())?;
        fs::rename(&temporary.0, path).map_err(|error| error.to_string())?;
        Ok(())
    };
    commit().map_err(|error| format!("failed to write file {}: {error}", path.display()))
}

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_save_protects_existing_bytes_at_every_failure_boundary() {
        let dir = std::env::temp_dir().join(format!("sgf-atomic-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("protected-sgf-中文.sgf");
        let original = b"original\0bytes\r\n";
        fs::write(&path, original).unwrap();
        let document = CurrentSgfDocument::open("(;SZ[9]C[new](;B[aa])(;B[bb]))").unwrap();
        for fault in [Stage::Prepare, Stage::Encode, Stage::Write, Stage::Replace] {
            let result = write_with(
                &path,
                || document.serialize().map_err(|error| error.to_string()),
                |stage| {
                    if std::mem::discriminant(&stage) == std::mem::discriminant(&fault) {
                        Err(io::Error::other(format!("{fault:?} fault")))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(result.unwrap_err().contains("failed to write"));
            assert_eq!(fs::read(&path).unwrap(), original);
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        }
        save_document_atomic(&document, &path).unwrap();
        let reopened = CurrentSgfDocument::open(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reopened.tree().unwrap(), document.tree().unwrap());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        let binary = b"captured\0image\xffbytes";
        write_file_atomic(&path, binary).unwrap();
        assert_eq!(fs::read(&path).unwrap(), binary);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_private_target_0600_remains_0600_after_sgf_and_bytes_writer() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("sgf-private-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("private.sgf");
        fs::write(&path, b"(;SZ[9]C[initial])").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);

        let document = CurrentSgfDocument::open("(;SZ[9]C[updated])").unwrap();
        save_document_atomic(&document, &path).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600,
            "private target 0600 must remain 0600 after SGF save"
        );
        let reopened = CurrentSgfDocument::open(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reopened.tree().unwrap(), document.tree().unwrap());

        let binary_payload = b"raw\0binary\xffbytes";
        write_file_atomic(&path, binary_payload).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600,
            "private target 0600 must remain 0600 after shared bytes writer"
        );
        assert_eq!(fs::read(&path).unwrap(), binary_payload);

        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_nondefault_permission_mode_preserved() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("sgf-nondefault-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();

        let path = dir.join("custom-mode.sgf");
        fs::write(&path, b"(;SZ[9]C[initial])").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);

        let document = CurrentSgfDocument::open("(;SZ[9]C[updated])").unwrap();
        save_document_atomic(&document, &path).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640,
            "nondefault mode 0640 must be preserved after SGF save"
        );

        let binary_payload = b"new\0image\x00data";
        write_file_atomic(&path, binary_payload).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640,
            "nondefault mode 0640 must be preserved after shared bytes writer"
        );
        assert_eq!(fs::read(&path).unwrap(), binary_payload);

        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_staging_mode_has_no_group_other_read_access_before_writing() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("sgf-staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("world-readable.sgf");
        fs::write(&path, b"(;SZ[9]C[initial])").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        let document = CurrentSgfDocument::open("(;SZ[9]C[updated])").unwrap();
        let observed_staging_mode = std::cell::RefCell::new(Vec::new());

        let result = write_with(
            &path,
            || document.serialize().map_err(|e| e.to_string()),
            |stage| {
                match stage {
                    Stage::Encode | Stage::Write => {
                        let entries = fs::read_dir(&dir).unwrap();
                        for entry in entries {
                            let entry = entry.unwrap();
                            if entry.path() != path {
                                let mode = entry.metadata().unwrap().permissions().mode();
                                observed_staging_mode.borrow_mut().push((stage, mode));
                            }
                        }
                    }
                    _ => {}
                }
                Ok(())
            },
        );

        assert!(result.is_ok());
        let observations = observed_staging_mode.into_inner();
        assert_eq!(
            observations.len(),
            2,
            "must observe staging file at Encode and Write stages"
        );
        for (stage, mode) in observations {
            assert_eq!(
                mode & 0o077,
                0,
                "{stage:?} staging file must have no group/other permissions"
            );
        }

        let final_mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            final_mode & 0o777,
            0o644,
            "final committed file must preserve destination 0644 mode"
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_new_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("sgf-new-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();

        let new_sgf = dir.join("new-document.sgf");
        assert!(!new_sgf.exists());
        let document = CurrentSgfDocument::open("(;SZ[9]C[new-doc])").unwrap();
        save_document_atomic(&document, &new_sgf).unwrap();
        let sgf_mode = fs::metadata(&new_sgf).unwrap().permissions().mode();
        assert_eq!(
            sgf_mode & 0o077,
            0,
            "new SGF file must have no group/other permissions"
        );

        let new_bin = dir.join("new-export.bin");
        assert!(!new_bin.exists());
        write_file_atomic(&new_bin, b"binary\0content").unwrap();
        let bin_mode = fs::metadata(&new_bin).unwrap().permissions().mode();
        assert_eq!(
            bin_mode & 0o077,
            0,
            "new binary file must have no group/other permissions"
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_target_bytes_and_mode_and_temp_cleanup_intact_at_failure_boundaries() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("sgf-faults-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("target-with-custom-mode.sgf");
        let original_bytes = b"original\0preserved\0bytes";
        fs::write(&path, original_bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        let document = CurrentSgfDocument::open("(;SZ[9]C[fault-doc])").unwrap();

        for fault in [Stage::Prepare, Stage::Encode, Stage::Write, Stage::Replace] {
            let result = write_with(
                &path,
                || document.serialize().map_err(|e| e.to_string()),
                |stage| {
                    if std::mem::discriminant(&stage) == std::mem::discriminant(&fault) {
                        Err(io::Error::other(format!("{fault:?} fault injected")))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(result.unwrap_err().contains("failed to write"));
            assert_eq!(
                fs::read(&path).unwrap(),
                original_bytes,
                "bytes must remain intact at {fault:?} boundary"
            );
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o640,
                "mode must remain intact at {fault:?} boundary"
            );
            assert_eq!(
                fs::read_dir(&dir).unwrap().count(),
                1,
                "temporary file must be cleaned up at {fault:?} boundary"
            );
        }

        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_non_not_found_metadata_error_propagates_before_commit() {
        use std::os::unix::fs::symlink;

        let dir = std::env::temp_dir().join(format!("sgf-meta-err-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let target = dir.join("loop.sgf");
        symlink("loop.sgf", &target).unwrap();

        let document = CurrentSgfDocument::open("(;SZ[9]C[test])").unwrap();
        let error = save_document_atomic(&document, &target).unwrap_err();
        assert!(error.contains("failed to write file"));
        assert_eq!(fs::read_link(&target).unwrap(), Path::new("loop.sgf"));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

        fs::remove_dir_all(dir).unwrap();
    }
}
