use rusqlite::{params, Connection};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameMetadata {
    pub id: String,
    pub source: String,
    pub source_id: Option<String>,
    pub board_size: i64,
    pub komi: f64,
    pub black_name: Option<String>,
    pub white_name: Option<String>,
    pub result: Option<String>,
    pub sgf_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameNode {
    pub id: String,
    pub game_id: String,
    pub parent_id: Option<String>,
    pub move_number: i64,
    pub color: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub comment: Option<String>,
    pub zobrist: Option<String>,
    pub sgf_path: Option<String>,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, name: "create_games", sql: "CREATE TABLE IF NOT EXISTS games (id TEXT PRIMARY KEY, source TEXT NOT NULL DEFAULT 'local', source_id TEXT, board_size INTEGER NOT NULL, komi REAL NOT NULL, black_name TEXT, white_name TEXT, result TEXT, sgf_hash TEXT, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP); CREATE INDEX IF NOT EXISTS idx_games_source ON games(source, source_id);" },
    Migration { version: 2, name: "create_game_nodes", sql: "CREATE TABLE IF NOT EXISTS game_nodes (id TEXT PRIMARY KEY, game_id TEXT NOT NULL, parent_id TEXT, move_number INTEGER NOT NULL, color TEXT, x INTEGER, y INTEGER, comment TEXT, zobrist TEXT, sgf_path TEXT, FOREIGN KEY(game_id) REFERENCES games(id)); CREATE INDEX IF NOT EXISTS idx_game_nodes_game_move ON game_nodes(game_id, move_number);" },
    Migration { version: 4, name: "create_engine_assets", sql: "CREATE TABLE IF NOT EXISTS engine_profiles (id TEXT PRIMARY KEY, name TEXT NOT NULL, engine_path TEXT NOT NULL, model_path TEXT, config_path TEXT, backend TEXT NOT NULL, config_json TEXT NOT NULL DEFAULT '{}'); CREATE TABLE IF NOT EXISTS assets (id TEXT PRIMARY KEY, kind TEXT NOT NULL, version TEXT, path TEXT NOT NULL, sha256 TEXT, installed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);" },
];

pub fn apply_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);")?;
    let tx = conn.transaction()?;
    for m in MIGRATIONS {
        let count: i64 = tx.query_row(
            "SELECT COUNT(1) FROM schema_migrations WHERE version = ?1",
            [m.version],
            |row| row.get(0),
        )?;
        if count == 0 {
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, name) VALUES(?1, ?2)",
                (m.version, m.name),
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn upsert_game_metadata(conn: &Connection, game: &GameMetadata) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO games (
            id, source, source_id, board_size, komi, black_name, white_name, result, sgf_hash
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
        )
        ON CONFLICT(id) DO UPDATE SET
            source = excluded.source,
            source_id = excluded.source_id,
            board_size = excluded.board_size,
            komi = excluded.komi,
            black_name = excluded.black_name,
            white_name = excluded.white_name,
            result = excluded.result,
            sgf_hash = excluded.sgf_hash,
            updated_at = CURRENT_TIMESTAMP",
        params![
            game.id,
            game.source,
            game.source_id,
            game.board_size,
            game.komi,
            game.black_name,
            game.white_name,
            game.result,
            game.sgf_hash
        ],
    )?;
    Ok(())
}

pub fn insert_game_node(conn: &Connection, node: &GameNode) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO game_nodes (
            id, game_id, parent_id, move_number, color, x, y, comment, zobrist, sgf_path
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
        )",
        params![
            node.id,
            node.game_id,
            node.parent_id,
            node.move_number,
            node.color,
            node.x,
            node.y,
            node.comment,
            node.zobrist,
            node.sgf_path
        ],
    )?;
    Ok(())
}

pub fn list_game_nodes(conn: &Connection, game_id: &str) -> Result<Vec<GameNode>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, game_id, parent_id, move_number, color, x, y, comment, zobrist, sgf_path
        FROM game_nodes
        WHERE game_id = ?1
        ORDER BY move_number ASC, id ASC",
    )?;
    let nodes = stmt
        .query_map([game_id], |row| {
            Ok(GameNode {
                id: row.get(0)?,
                game_id: row.get(1)?,
                parent_id: row.get(2)?,
                move_number: row.get(3)?,
                color: row.get(4)?,
                x: row.get(5)?,
                y: row.get(6)?,
                comment: row.get(7)?,
                zobrist: row.get(8)?,
                sgf_path: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn migrated_connection() -> Connection {
        let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
        apply_migrations(&mut conn).expect("apply migrations");
        conn
    }

    fn sample_game() -> GameMetadata {
        GameMetadata {
            id: "game-1".to_string(),
            source: "local".to_string(),
            source_id: Some("sgf-1".to_string()),
            board_size: 19,
            komi: 7.5,
            black_name: Some("Black".to_string()),
            white_name: Some("White".to_string()),
            result: Some("B+R".to_string()),
            sgf_hash: Some("hash-1".to_string()),
        }
    }

    #[test]
    fn apply_migrations_is_idempotent_for_in_memory_database() {
        let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");

        apply_migrations(&mut conn).expect("first migration run");
        apply_migrations(&mut conn).expect("second migration run");

        let applied_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM schema_migrations", [], |row| row.get(0))
            .expect("count applied migrations");
        assert_eq!(applied_count, MIGRATIONS.len() as i64);
    }

    #[test]
    fn apply_migrations_enables_foreign_key_enforcement() {
        let conn = migrated_connection();

        let foreign_keys_enabled: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("read foreign_keys pragma");
        assert_eq!(foreign_keys_enabled, 1);

        let orphan_node = GameNode {
            id: "orphan-node".to_string(),
            game_id: "missing-game".to_string(),
            parent_id: None,
            move_number: 0,
            color: None,
            x: None,
            y: None,
            comment: None,
            zobrist: None,
            sgf_path: None,
        };
        assert!(insert_game_node(&conn, &orphan_node).is_err());
    }

    #[test]
    fn upserts_game_metadata() {
        let conn = migrated_connection();
        let mut game = sample_game();

        upsert_game_metadata(&conn, &game).expect("insert game");
        game.black_name = Some("Updated Black".to_string());
        game.result = Some("W+2.5".to_string());
        upsert_game_metadata(&conn, &game).expect("update game");

        let stored: (String, String, Option<String>) = conn
            .query_row(
                "SELECT id, black_name, result FROM games WHERE id = ?1",
                ["game-1"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("load game");
        assert_eq!(
            stored,
            (
                "game-1".to_string(),
                "Updated Black".to_string(),
                Some("W+2.5".to_string())
            )
        );
    }

    #[test]
    fn inserts_and_lists_game_nodes() {
        let conn = migrated_connection();
        upsert_game_metadata(&conn, &sample_game()).expect("insert game");

        let root = GameNode {
            id: "node-1".to_string(),
            game_id: "game-1".to_string(),
            parent_id: None,
            move_number: 0,
            color: None,
            x: None,
            y: None,
            comment: Some("root".to_string()),
            zobrist: Some("z0".to_string()),
            sgf_path: Some(";".to_string()),
        };
        let move_one = GameNode {
            id: "node-2".to_string(),
            game_id: "game-1".to_string(),
            parent_id: Some("node-1".to_string()),
            move_number: 1,
            color: Some("B".to_string()),
            x: Some(3),
            y: Some(3),
            comment: None,
            zobrist: Some("z1".to_string()),
            sgf_path: Some(";B[dd]".to_string()),
        };
        insert_game_node(&conn, &move_one).expect("insert move one");
        insert_game_node(&conn, &root).expect("insert root");

        let nodes = list_game_nodes(&conn, "game-1").expect("list nodes");
        assert_eq!(nodes, vec![root, move_one]);
    }
}
