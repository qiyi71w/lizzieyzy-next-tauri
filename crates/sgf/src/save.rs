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
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = parent.join(format!(".lizzieyzy-sgf-{}.tmp", uuid::Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| error.to_string())?;
        let temporary = Temporary(temporary);
        let prepared = (|| -> Result<(), String> {
            before(Stage::Encode).map_err(|error| error.to_string())?;
            let contents = encode()?;
            before(Stage::Write).map_err(|error| error.to_string())?;
            file.write_all(contents.as_ref())
                .map_err(|error| error.to_string())?;
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
}
