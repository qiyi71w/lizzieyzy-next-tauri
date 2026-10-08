use app_model::{
    EngineBackend, EngineRunDto, ExactPositionDto, ExactRulesDto, MoveVertex, PlayerColor, StoneDto,
};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

pub(super) fn admit(run: &EngineRunDto, position: &sgf::ExactPosition) -> Result<(), String> {
    if run.adapter_kind != EngineBackend::KataGoGtp || run.qualified_resource.is_none() {
        return Err("Ordinary rules confirmation requires a qualified explicit KataGo GTP Run".into());
    }
    let facts = run
        .capability_snapshot
        .as_ref()
        .and_then(|caps| caps.gtp.as_ref())
        .ok_or("Actual GTP identity is unavailable")?;
    if facts.name.trim() != "KataGo" || facts.version.split('+').next() != Some("1.18.2") {
        return Err("This KataGo version has not qualified exact rules confirmation".into());
    }
    if position.dto().komi.abs() > 400.0 {
        return Err("KataGo exact komi must be in [-400,400]".into());
    }
    Ok(())
}

/// KataGo 1.18.2 splits loadsgf arguments on whitespace, including inside quotes.
/// Send only our private ASCII basename relative to the captured process cwd.
/// No user path or SGF text is ever interpolated into a GTP command.
pub(super) struct RestoreFile {
    pub basename: String,
    path: PathBuf,
}
impl RestoreFile {
    pub fn create(run: &EngineRunDto, position: &ExactPositionDto) -> Result<Self, String> {
        let basename = format!(".lizzie-rules-{}.sgf", uuid::Uuid::new_v4());
        let cwd = match run
            .profile_snapshot
            .working_dir
            .as_deref()
            .filter(|s| !s.trim().is_empty())
        {
            Some(path) => PathBuf::from(path),
            None => std::env::current_dir().map_err(|_| "Cannot resolve the engine working directory")?,
        };
        let path = cwd.join(&basename);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .map_err(|_| "Cannot create a private restore file in the engine working directory")?;
        let restore = Self { basename, path };
        file.write_all(serialize(position).as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| "Cannot publish the private exact-position restore file")?;
        File::open(&restore.path).map_err(|_| "Private restore file is not readable")?;
        Ok(restore)
    }
    pub fn remove(self) -> Result<(), String> {
        std::fs::remove_file(&self.path).map_err(|_| "Private restore file cleanup failed".into())
    }
}
impl Drop for RestoreFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn color(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Black => "B",
        PlayerColor::White => "W",
    }
}
fn vertex(vertex: &MoveVertex) -> String {
    match vertex {
        MoveVertex::Pass => String::new(),
        MoveVertex::Point(point) => format!("{}{}", char::from(b'a' + point.x), char::from(b'a' + point.y)),
    }
}
fn serialize(position: &ExactPositionDto) -> String {
    let rules = match position.rules {
        ExactRulesDto::Chinese => "Chinese",
        ExactRulesDto::ChineseKgs => "Chinese-KGS",
    };
    let mut text = format!(
        "(;FF[4]GM[1]SZ[{}]RU[{}]KM[{}]PL[{}]",
        position.board_width,
        rules,
        position.komi,
        color(position.initial_player)
    );
    if !position.initial_stones.is_empty() {
        text.push_str(&format!("HA[{}]AB", position.initial_stones.len()));
        for stone in &position.initial_stones {
            text.push_str(&format!(
                "[{}{}]",
                char::from(b'a' + stone.x),
                char::from(b'a' + stone.y)
            ));
        }
    }
    for played in &position.moves {
        text.push_str(&format!(";{}[{}]", color(played.color), vertex(&played.vertex)));
    }
    text.push(')');
    text
}

pub(super) fn verify(
    position: &sgf::ExactPosition,
    rules: &str,
    komi: &str,
    board: &str,
    sgf_text: &str,
) -> Result<serde_json::Value, String> {
    let dto = position.dto();
    let actual: serde_json::Value =
        serde_json::from_str(rules).map_err(|_| "Invalid actual rules response")?;
    let expected = serde_json::json!({"friendlyPassOk":true,"hasButton":false,
        "ko": match dto.rules { ExactRulesDto::Chinese => "SIMPLE", ExactRulesDto::ChineseKgs => "POSITIONAL" },
        "scoring":"AREA","suicide":false,"tax":"NONE","whiteHandicapBonus":"N"});
    if actual != expected {
        return Err("Actual engine rules do not exactly match requested rules".into());
    }
    if komi.trim().parse::<f32>().ok() != Some(dto.komi) {
        return Err("Actual engine komi does not match requested komi".into());
    }
    let expected_player = match dto.to_play {
        PlayerColor::Black => "Next player: Black",
        PlayerColor::White => "Next player: White",
    };
    if board
        .lines()
        .filter(|line| line.starts_with("Next player:"))
        .collect::<Vec<_>>()
        != [expected_player]
    {
        return Err("Actual engine player to play differs from the selected position".into());
    }
    let mut stones = Vec::new();
    let mut rows = vec![false; usize::from(dto.board_height)];
    for line in board.lines() {
        let mut fields = line.split_whitespace();
        let Some(row) = fields.next().and_then(|s| s.parse::<u8>().ok()) else {
            continue;
        };
        if row == 0 || row > dto.board_height || rows[usize::from(row - 1)] {
            return Err("Invalid or duplicated actual board row".into());
        }
        rows[usize::from(row - 1)] = true;
        // showboard places recent move digits directly after stones (e.g. X1.).
        let points: Vec<_> = fields
            .flat_map(str::chars)
            .filter(|c| !c.is_ascii_digit())
            .collect();
        if points.len() != usize::from(dto.board_width) {
            return Err("Actual board width mismatch".into());
        }
        for (x, point) in points.into_iter().enumerate() {
            let color = match point {
                '.' | '+' => continue,
                'X' => PlayerColor::Black,
                'O' => PlayerColor::White,
                _ => return Err("Invalid actual board stone".into()),
            };
            stones.push(StoneDto {
                x: x as u8,
                y: dto.board_height - row,
                color,
            });
        }
    }
    let mut expected_stones = position.stones();
    stones.sort_by_key(|stone| (stone.y, stone.x));
    expected_stones.sort_by_key(|stone| (stone.y, stone.x));
    if rows.contains(&false) || stones != expected_stones {
        return Err("Actual engine stones differ from the selected position".into());
    }
    let document =
        sgf::CurrentSgfDocument::open(sgf_text).map_err(|_| "Engine returned invalid history SGF")?;
    if document.mainline_projection().moves != dto.moves {
        return Err(
            "Actual engine move history or true final move differs from the selected position".into(),
        );
    }
    Ok(actual)
}
