//! Local content descriptions and retained paths. This module never admits an Engine Run.
use app_model::{ModelInspectionDto, ModelInspectionStatusDto as Status};
use flate2::read::MultiGzDecoder;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::time::{Duration, Instant};

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_DECODED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const HEADER_BYTES: usize = 16 * 1024;
const INSPECTION_TIME: Duration = Duration::from_secs(30);

fn empty_inspection(status: Status) -> ModelInspectionDto {
    ModelInspectionDto {
        status,
        sha256: None,
        size_bytes: None,
        format: None,
        model_name: None,
        format_version: None,
    }
}

struct ContentReader<R> {
    reader: R,
    hash: Sha256,
    bytes: u64,
    deadline: Instant,
}
impl<R: Read> Read for ContentReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if Instant::now() >= self.deadline || self.bytes > MAX_FILE_BYTES {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "model inspection bound"));
        }
        let count = self.reader.read(buffer)?;
        self.hash.update(&buffer[..count]);
        self.bytes += count as u64;
        Ok(count)
    }
}

/// Full compressed-file SHA256 and gzip CRC, plus a bounded header description.
/// Recognizing a header does not validate all network tensors or engine compatibility.
/// The deadline is cooperative between regular-file reads, not OS I/O cancellation.
pub fn inspect_model(path: &Path) -> ModelInspectionDto {
    inspect_until(path, Instant::now() + INSPECTION_TIME)
}

fn inspect_until(path: &Path, deadline: Instant) -> ModelInspectionDto {
    let Ok(metadata) = std::fs::metadata(path) else {
        return empty_inspection(Status::Unavailable);
    };
    if !metadata.is_file() {
        return empty_inspection(Status::Unavailable);
    }
    if metadata.len() > MAX_FILE_BYTES {
        return empty_inspection(Status::LimitExceeded);
    }
    let Ok(file) = File::open(path) else {
        return empty_inspection(Status::Unavailable);
    };
    let mut content = ContentReader {
        reader: file,
        hash: Sha256::new(),
        bytes: 0,
        deadline,
    };
    let mut reader = BufReader::new(&mut content);
    let gzip = match reader.fill_buf() {
        Ok(bytes) => bytes.starts_with(&[0x1f, 0x8b]),
        Err(_) => return empty_inspection(Status::Unavailable),
    };
    let mut prefix = Vec::with_capacity(HEADER_BYTES);
    let result = if gzip {
        read_content(&mut MultiGzDecoder::new(reader), &mut prefix, deadline)
    } else {
        read_content(&mut reader, &mut prefix, deadline)
    };
    let mut inspection = empty_inspection(Status::Unknown);
    inspection.size_bytes = Some(metadata.len());
    match result {
        Err(error) => {
            inspection.status = if error.kind() == io::ErrorKind::TimedOut {
                Status::LimitExceeded
            } else if gzip {
                Status::Corrupt
            } else {
                Status::Unavailable
            };
            return inspection;
        }
        Ok(()) => inspection.sha256 = Some(format!("{:x}", content.hash.finalize())),
    }
    if !std::fs::metadata(path)
        .is_ok_and(|after| after.len() == metadata.len() && after.modified().ok() == metadata.modified().ok())
    {
        inspection.status = Status::Changed;
        return inspection;
    }
    describe_header(&prefix, gzip, &mut inspection);
    inspection
}

fn read_content(reader: &mut impl Read, prefix: &mut Vec<u8>, deadline: Instant) -> io::Result<()> {
    let mut buffer = [0u8; 64 * 1024];
    let mut decoded = 0u64;
    loop {
        if Instant::now() >= deadline || decoded > MAX_DECODED_BYTES {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "model inspection bound"));
        }
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(());
        }
        decoded += count as u64;
        let keep = count.min(HEADER_BYTES - prefix.len());
        prefix.extend_from_slice(&buffer[..keep]);
    }
}

fn describe_header(prefix: &[u8], gzip: bool, result: &mut ModelInspectionDto) {
    // KataGo desc.cpp at 47aadc08518b3e121f22539796c911002f699584:
    // name, format version, spatial/global channels; binary weights start @BIN@.
    let binary = prefix.windows(5).position(|bytes| bytes == b"@BIN@");
    let text = match std::str::from_utf8(&prefix[..binary.unwrap_or(prefix.len())]) {
        Ok(text) => text,
        Err(_) => return,
    };
    let mut tokens = text.split_ascii_whitespace();
    let Some(name) = tokens.next() else { return };
    if name.len() > 256 || !name.bytes().all(|byte| byte.is_ascii_graphic()) {
        return;
    }
    let Some(version) = tokens.next().and_then(|value| value.parse::<u32>().ok()) else {
        return;
    };
    if !(3..=17).contains(&version) {
        return;
    }
    result.status = Status::Corrupt;
    for _ in 0..2 {
        if !tokens
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|value| value > 0 && value <= 4096)
        {
            return;
        }
    }
    if version >= 13 {
        for _ in 0..7 {
            if !tokens
                .next()
                .and_then(|value| value.parse::<f64>().ok())
                .is_some_and(|value| value.is_finite() && value > 0.0)
            {
                return;
            }
        }
    }
    // Require the network trunk and a weight encoding, without pretending to validate tensors.
    if !tokens.any(|value| value == "trunk") {
        return;
    }
    if binary.is_none() {
        // Text weights are a supported upstream format, but are not qualified by our samples.
        result.status = Status::Unknown;
        return;
    }
    result.status = Status::HeaderRecognized;
    result.model_name = Some(name.to_owned());
    result.format_version = Some(version);
    result.format = Some(
        if gzip {
            "katago_binary_gzip"
        } else {
            "katago_binary"
        }
        .into(),
    );
}

use app_model::{
    InstalledModelDto, ModelInventoryDto, ModelOriginDto, ModelPathDto, ModelSelectionRequestDto,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

const MAX_MODELS: usize = 128;
const MAX_INVENTORY_BYTES: u64 = 1024 * 1024;
const REFRESH_TIME: Duration = Duration::from_secs(60);

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedModel {
    id: String,
    path: String,
    origin: ModelOriginDto,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryFile<T> {
    version: u32,
    models: T,
}

/// One non-queuing owner for local inspection and atomic retained-path updates.
/// Saved profile selection remains exclusively in the profile catalog.
pub struct ModelInventory {
    path: PathBuf,
    snapshot: Mutex<Option<ModelInventoryDto>>,
}

impl ModelInventory {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            snapshot: Mutex::new(None),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, Option<ModelInventoryDto>>, String> {
        self.snapshot
            .try_lock()
            .map_err(|_| "model_inventory_busy".into())
    }

    fn load(&self) -> Result<Vec<RetainedModel>, String> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err("model_inventory_read_failed".into()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_INVENTORY_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "model_inventory_read_failed")?;
        if bytes.len() as u64 > MAX_INVENTORY_BYTES {
            return Err("model_inventory_limit".into());
        }
        let file: InventoryFile<Vec<RetainedModel>> =
            serde_json::from_slice(&bytes).map_err(|_| "model_inventory_corrupt")?;
        let mut ids = std::collections::HashSet::new();
        let mut paths = std::collections::HashSet::new();
        if file.version != 1
            || file.models.len() > MAX_MODELS
            || file.models.iter().any(|model| {
                model.id.is_empty()
                    || model.path.is_empty()
                    || !Path::new(&model.path).is_absolute()
                    || !ids.insert(&model.id)
                    || !paths.insert(&model.path)
            })
        {
            return Err("model_inventory_corrupt".into());
        }
        Ok(file.models)
    }

    fn save(&self, models: &[RetainedModel]) -> Result<(), String> {
        let bytes = serde_json::to_vec(&InventoryFile { version: 1, models })
            .map_err(|_| "model_inventory_write_failed")?;
        if bytes.len() as u64 > MAX_INVENTORY_BYTES {
            return Err("model_inventory_limit".into());
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| "model_inventory_write_failed")?;
        }
        sgf::write_file_atomic(&self.path, &bytes).map_err(|_| "model_inventory_write_failed".into())
    }

    /// Retain saved paths without scanning or changing their serialized launch inputs.
    /// Imported legacy paths have unknown ownership; names never confer managed origin.
    pub fn remember_saved(&self, paths: &[ModelPathDto]) -> Result<(), String> {
        let mut snapshot = self.lock()?;
        let mut models = self.load()?;
        if merge_paths(&mut models, paths, ModelOriginDto::Unknown)? {
            self.save(&models)?;
            *snapshot = None;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<ModelInventoryDto, String> {
        let mut snapshot = self.lock()?;
        if let Some(snapshot) = snapshot.as_ref() {
            return Ok(snapshot.clone());
        }
        let result = make_snapshot(self.load()?);
        *snapshot = Some(result.clone());
        Ok(result)
    }

    /// Explicit refresh retains custom inputs even if missing, unknown or corrupt.
    /// It never selects a model, edits a profile, downloads or starts an engine.
    pub fn refresh(&self, paths: &[ModelPathDto]) -> Result<ModelInventoryDto, String> {
        let mut snapshot = self.lock()?;
        // A failed/unfinished refresh cannot leave an old snapshot selectable.
        *snapshot = None;
        let mut models = self.load()?;
        if merge_paths(&mut models, paths, ModelOriginDto::Custom)? {
            self.save(&models)?;
        }
        let mut result = make_snapshot(models);
        let deadline = Instant::now() + REFRESH_TIME;
        for model in &mut result.models {
            model.inspection = inspect_until(
                Path::new(&model.path),
                deadline.min(Instant::now() + INSPECTION_TIME),
            );
        }
        *snapshot = Some(result.clone());
        Ok(result)
    }

    /// Called by the managed installer only after its own catalog/source admission.
    /// This receipt records installation ownership, not ongoing resource trust.
    pub fn retain_managed(
        &self,
        path: &Path,
        catalog_id: &str,
        installed_sha256: &str,
    ) -> Result<(), String> {
        let mut snapshot = self.lock()?;
        let inspection = inspect_model(path);
        if catalog_id.is_empty() || inspection.sha256.as_deref() != Some(installed_sha256) {
            return Err("model_installation_changed".into());
        }
        let input = ModelPathDto {
            path: path.to_string_lossy().into_owned(),
            working_dir: None,
        };
        let resolved = resolve_model_path(&input)?;
        let mut models = self.load()?;
        merge_paths(&mut models, &[input], ModelOriginDto::Unknown)?;
        let model = models
            .iter_mut()
            .find(|model| model.path == resolved)
            .expect("retained model");
        model.origin = ModelOriginDto::Managed {
            catalog_id: catalog_id.to_owned(),
            installed_sha256: installed_sha256.to_owned(),
        };
        self.save(&models)?;
        *snapshot = None;
        Ok(())
    }

    /// Explicit draft selection re-reads content, including same-size/same-time replacements.
    /// A path receipt is not engine admission; Start still qualifies the actual launch.
    pub fn select(&self, request: &ModelSelectionRequestDto) -> Result<String, String> {
        let mut snapshot = self.lock()?;
        let current = snapshot
            .as_mut()
            .filter(|snapshot| snapshot.revision == request.revision)
            .ok_or("model_snapshot_stale")?;
        let model = current
            .models
            .iter_mut()
            .find(|model| model.id == request.model_id)
            .ok_or("model_snapshot_stale")?;
        if model.inspection.status != Status::HeaderRecognized
            || model.inspection.sha256.as_deref() != Some(&request.sha256)
        {
            return Err("model_not_selectable".into());
        }
        let inspected = inspect_model(Path::new(&model.path));
        if inspected.status != Status::HeaderRecognized || inspected.sha256 != model.inspection.sha256 {
            model.inspection = inspected;
            current.revision = uuid::Uuid::new_v4().to_string();
            return Err("model_content_changed_refresh_required".into());
        }
        Ok(model.path.clone())
    }
}

fn resolve_model_path(input: &ModelPathDto) -> Result<String, String> {
    if input.path.is_empty() || input.path.len() > 8192 || input.path.contains('\0') {
        return Err("model_path_invalid".into());
    }
    let path = Path::new(&input.path);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(directory) = input
        .working_dir
        .as_ref()
        .filter(|directory| !directory.trim().is_empty())
    {
        let directory = Path::new(directory);
        if directory.is_absolute() {
            directory.join(path)
        } else {
            std::env::current_dir()
                .map_err(|_| "model_path_invalid")?
                .join(directory)
                .join(path)
        }
    } else {
        std::env::current_dir()
            .map_err(|_| "model_path_invalid")?
            .join(path)
    };
    // Keep the resolved spelling, including symlinks: replacing that path must be re-inspected.
    Ok(resolved.to_string_lossy().into_owned())
}

fn merge_paths(
    models: &mut Vec<RetainedModel>,
    paths: &[ModelPathDto],
    origin: ModelOriginDto,
) -> Result<bool, String> {
    if paths.len() > MAX_MODELS * 2 {
        return Err("model_inventory_limit".into());
    }
    let mut changed = false;
    for path in paths {
        if path.path.is_empty() {
            continue;
        }
        let path = resolve_model_path(path)?;
        if models.iter().any(|model| model.path == path) {
            continue;
        }
        if models.len() == MAX_MODELS {
            return Err("model_inventory_limit".into());
        }
        models.push(RetainedModel {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            origin: origin.clone(),
        });
        changed = true;
    }
    Ok(changed)
}

fn make_snapshot(models: Vec<RetainedModel>) -> ModelInventoryDto {
    ModelInventoryDto {
        revision: uuid::Uuid::new_v4().to_string(),
        models: models
            .into_iter()
            .map(|model| InstalledModelDto {
                id: model.id,
                path: model.path,
                origin: model.origin,
                inspection: empty_inspection(Status::Unchecked),
            })
            .collect(),
    }
}
