use rusqlite::Connection;
use storage::{apply_migrations, upsert_game_metadata, GameMetadata, Migration};

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .unwrap()
        > 0
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
fn fresh_database_has_no_analysis_cache_tables() {
    let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
    apply_migrations(&mut conn).expect("apply migrations");
    assert!(!table_exists(&conn, "analysis_jobs"));
    assert!(!table_exists(&conn, "analysis_positions"));
    assert!(table_exists(&conn, "games"));
    assert!(table_exists(&conn, "game_nodes"));
    assert!(table_exists(&conn, "engine_profiles"));
    assert!(table_exists(&conn, "assets"));
}

#[test]
fn existing_development_database_with_analysis_tables_still_starts() {
    let mut conn = Connection::open_in_memory().expect("open in-memory sqlite");
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);",
    )
    .unwrap();
    for migration in [
        Migration {
            version: 1,
            name: "create_games",
            sql: "CREATE TABLE IF NOT EXISTS games (id TEXT PRIMARY KEY, source TEXT NOT NULL DEFAULT 'local', source_id TEXT, board_size INTEGER NOT NULL, komi REAL NOT NULL, black_name TEXT, white_name TEXT, result TEXT, sgf_hash TEXT, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);",
        },
        Migration {
            version: 2,
            name: "create_game_nodes",
            sql: "CREATE TABLE IF NOT EXISTS game_nodes (id TEXT PRIMARY KEY, game_id TEXT NOT NULL, parent_id TEXT, move_number INTEGER NOT NULL, color TEXT, x INTEGER, y INTEGER, comment TEXT, zobrist TEXT, sgf_path TEXT, FOREIGN KEY(game_id) REFERENCES games(id));",
        },
        Migration {
            version: 3,
            name: "create_analysis",
            sql: "CREATE TABLE IF NOT EXISTS analysis_jobs (id TEXT PRIMARY KEY, game_id TEXT, engine_profile_id TEXT, model_hash TEXT, visits INTEGER NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, finished_at TEXT, FOREIGN KEY(game_id) REFERENCES games(id)); CREATE TABLE IF NOT EXISTS analysis_positions (id TEXT PRIMARY KEY, job_id TEXT NOT NULL, node_id TEXT, turn INTEGER NOT NULL, visits INTEGER NOT NULL, winrate_black REAL NOT NULL, score_mean_black REAL NOT NULL, score_stdev REAL, policy_json TEXT, ownership_json TEXT, candidates_json TEXT NOT NULL, raw_json TEXT, FOREIGN KEY(job_id) REFERENCES analysis_jobs(id));",
        },
    ] {
        conn.execute_batch(migration.sql).unwrap();
        conn.execute(
            "INSERT INTO schema_migrations(version, name) VALUES(?1, ?2)",
            (migration.version, migration.name),
        )
        .unwrap();
    }
    upsert_game_metadata(&conn, &sample_game()).expect("seed unrelated game row");

    apply_migrations(&mut conn).expect("existing development database starts");

    let count: i64 = conn
        .query_row("SELECT COUNT(1) FROM games WHERE id = 'game-1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
    assert!(table_exists(&conn, "engine_profiles"));
    assert!(table_exists(&conn, "assets"));
    upsert_game_metadata(&conn, &sample_game()).expect("unrelated game schema remains writable");
}
