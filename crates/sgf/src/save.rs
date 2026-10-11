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

/// Cancellation and publication share this gate. New output never replaces an existing file.
pub fn save_document_authorized(
    document: &CurrentSgfDocument,
    path: &Path,
    overwrite: bool,
    cancelled: &parking_lot::Mutex<bool>,
) -> Result<(), String> {
    write_with_publication(
        path,
        || document.serialize().map_err(|e| e.to_string()),
        |_| Ok(()),
        |temporary, target| {
            let gate = cancelled.lock();
            if *gate {
                return Err("Output cancelled before atomic publication".into());
            }
            if overwrite {
                fs::rename(temporary, target)
            } else {
                publish_noreplace(temporary, target)
            }
            .map_err(|e| e.to_string())
        },
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn publish_noreplace(temporary: &Path, target: &Path) -> io::Result<()> {
    use rustix::fs::{renameat_with, RenameFlags, CWD};
    publish_with_link_fallback(temporary, target, || {
        renameat_with(CWD, temporary, CWD, target, RenameFlags::NOREPLACE)
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn publish_with_link_fallback(
    temporary: &Path,
    target: &Path,
    rename: impl FnOnce() -> Result<(), rustix::io::Errno>,
) -> io::Result<()> {
    match rename() {
        // Some kernels/filesystems (including WSL DrvFS) lack exclusive rename.
        // Hard linking still publishes the complete staged bytes without replacing.
        Err(rustix::io::Errno::NOSYS | rustix::io::Errno::INVAL | rustix::io::Errno::OPNOTSUPP) => {
            fs::hard_link(temporary, target)
        }
        result => result.map_err(Into::into),
    }
}

#[cfg(windows)]
fn publish_noreplace(temporary: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    // Canonicalizing the shared parent supplies an extended-length absolute path
    // without following an existing destination file or changing its identity.
    let parent = fs::canonicalize(temporary.parent().unwrap_or(Path::new(".")))?;
    let wide = |path: &Path| -> io::Result<Vec<u16>> {
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Output needs a file name"))?;
        let mut value: Vec<u16> = parent.join(name).as_os_str().encode_wide().collect();
        if value.contains(&0) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Path contains NUL"));
        }
        value.push(0);
        Ok(value)
    };
    let source = wide(temporary)?;
    let destination = wide(target)?;
    // Both NUL-terminated paths live through this synchronous call. Zero flags
    // prohibit replacement and cross-volume copy/delete fallback.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn publish_noreplace(temporary: &Path, target: &Path) -> io::Result<()> {
    fs::hard_link(temporary, target)
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
    write_with_publication(path, encode, before, |temporary, target| {
        fs::rename(temporary, target).map_err(|e| e.to_string())
    })
}

fn write_with_publication<T: AsRef<[u8]>>(
    path: &Path,
    encode: impl FnOnce() -> Result<T, String>,
    before: impl Fn(Stage) -> io::Result<()>,
    publish: impl FnOnce(&Path, &Path) -> Result<(), String>,
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
        publish(&temporary.0, path)?;
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
    fn authorized_publishers_have_one_complete_winner_and_preserve_directories() {
        let dir = std::env::temp_dir().join(format!("batch-race-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let target = dir.join("result-中文.sgf");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let writers: Vec<_> = ["(;SZ[9]C[first])", "(;SZ[13]C[second])"]
            .into_iter()
            .map(|text| {
                let target = target.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let document = CurrentSgfDocument::open(text).unwrap();
                    let expected = document.serialize().unwrap();
                    barrier.wait();
                    (
                        save_document_authorized(&document, &target, false, &parking_lot::Mutex::new(false)),
                        expected,
                    )
                })
            })
            .collect();
        let results: Vec<_> = writers.into_iter().map(|writer| writer.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|(result, _)| result.is_ok()).count(), 1);
        let winner = &results.iter().find(|(result, _)| result.is_ok()).unwrap().1;
        assert_eq!(fs::read_to_string(&target).unwrap(), *winner);
        CurrentSgfDocument::open(winner).unwrap();
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_file(&target).unwrap();
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"protected").unwrap();
        let document = CurrentSgfDocument::open("(;SZ[9])").unwrap();
        assert!(
            save_document_authorized(&document, &target, false, &parking_lot::Mutex::new(false)).is_err()
        );
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"protected");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn unavailable_exclusive_rename_uses_atomic_link_but_other_errors_do_not() {
        use rustix::io::Errno;
        let dir = std::env::temp_dir().join(format!("batch-fallback-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let target = dir.join("result.sgf");
        for error in [Errno::NOSYS, Errno::INVAL, Errno::OPNOTSUPP] {
            let publish = |temporary: &Path, target: &Path| {
                publish_with_link_fallback(temporary, target, || Err(error)).map_err(|e| e.to_string())
            };
            write_with_publication(&target, || Ok(b"complete"), |_| Ok(()), publish).unwrap();
            assert_eq!(fs::read(&target).unwrap(), b"complete");
            assert!(write_with_publication(&target, || Ok(b"loser"), |_| Ok(()), publish).is_err());
            assert_eq!(fs::read(&target).unwrap(), b"complete");
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
            fs::remove_file(&target).unwrap();
        }
        for error in [Errno::EXIST, Errno::ACCESS, Errno::PERM, Errno::IO, Errno::XDEV] {
            let result = write_with_publication(
                &target,
                || Ok(b"must not publish"),
                |_| Ok(()),
                |temporary, target| {
                    publish_with_link_fallback(temporary, target, || Err(error)).map_err(|e| e.to_string())
                },
            );
            assert!(result.is_err());
            assert!(!target.exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn authorized_output_linearizes_cancel_and_never_clobbers_without_permission() {
        let dir = std::env::temp_dir().join(format!("batch-publication-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("result.sgf");
        let gate = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let mut held = gate.lock();
        let writer_gate = gate.clone();
        let writer_path = path.clone();
        let writer = std::thread::spawn(move || {
            let document = CurrentSgfDocument::open("(;SZ[9]C[personal])").unwrap();
            save_document_authorized(&document, &writer_path, false, &writer_gate)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while fs::read_dir(&dir).unwrap().count() == 0 {
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        *held = true;
        drop(held);
        assert!(writer.join().unwrap().unwrap_err().contains("cancelled"));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        *gate.lock() = false;
        let document = CurrentSgfDocument::open("(;SZ[9]C[personal])").unwrap();
        save_document_authorized(&document, &path, false, &gate).unwrap();
        let committed = fs::read(&path).unwrap();
        *gate.lock() = true;
        assert_eq!(fs::read(&path).unwrap(), committed);
        *gate.lock() = false;
        let changed = CurrentSgfDocument::open("(;SZ[13])").unwrap();
        assert!(save_document_authorized(&changed, &path, false, &gate).is_err());
        assert_eq!(fs::read(&path).unwrap(), committed);
        save_document_authorized(&changed, &path, true, &gate).unwrap();
        assert_eq!(
            CurrentSgfDocument::open(&fs::read_to_string(&path).unwrap())
                .unwrap()
                .tree()
                .unwrap(),
            changed.tree().unwrap()
        );
        fs::remove_dir_all(dir).unwrap();
    }

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
