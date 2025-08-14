use std::io;
use rusqlite::{Connection, Result as SqlResult, params};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{Board, column::Column, task::{Task, Priority}};

const DB_FILE: &str = "kanban_board.db";

pub fn init_database() -> SqlResult<Connection> {
    let conn = Connection::open(DB_FILE)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS boards (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS columns (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            board_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            FOREIGN KEY(board_id) REFERENCES boards(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            due_date TEXT,
            priority TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            column_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            FOREIGN KEY(column_id) REFERENCES columns(id)
        )",
        [],
    )?;

    Ok(conn)
}

pub fn save_board(board: &Board) -> io::Result<()> {
    let conn = init_database()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    conn.execute("DELETE FROM tasks", [])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;
    conn.execute("DELETE FROM columns", [])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;
    conn.execute("DELETE FROM boards", [])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    conn.execute(
        "INSERT INTO boards (id, title) VALUES (?1, ?2)",
        params![board.id.to_string(), board.title],
    ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    for (col_pos, column) in board.columns.iter().enumerate() {
        conn.execute(
            "INSERT INTO columns (id, title, board_id, position) VALUES (?1, ?2, ?3, ?4)",
            params![column.id.to_string(), column.title, board.id.to_string(), col_pos as i32],
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

        for (task_pos, task) in column.tasks.iter().enumerate() {
            let priority_str = match task.priority {
                Priority::Low => "Low",
                Priority::Medium => "Medium", 
                Priority::High => "High",
                Priority::Critical => "Critical",
            };

            conn.execute(
                "INSERT INTO tasks (id, title, description, due_date, priority, created_at, updated_at, column_id, position) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    task.id.to_string(),
                    task.title,
                    task.description,
                    task.due_date.map(|d| d.to_rfc3339()),
                    priority_str,
                    task.created_at.to_rfc3339(),
                    task.updated_at.to_rfc3339(),
                    column.id.to_string(),
                    task_pos as i32
                ],
            ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;
        }
    }

    Ok(())
}

pub fn load_board() -> io::Result<Board> {
    let conn = init_database()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    let mut stmt = conn.prepare("SELECT id, title FROM boards LIMIT 1")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;
    
    let board_row = stmt.query_row([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    });

    let (board_id, board_title) = match board_row {
        Ok((id, title)) => (id, title),
        Err(_) => {
            let new_board = Board::new("My Kanban Board".to_string());
            save_board(&new_board)?;
            return Ok(new_board);
        }
    };

    let board_uuid = Uuid::parse_str(&board_id)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UUID: {}", e)))?;

    let mut columns_stmt = conn.prepare(
        "SELECT id, title FROM columns WHERE board_id = ?1 ORDER BY position"
    ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    let column_rows = columns_stmt.query_map([&board_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    }).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    let mut columns = Vec::new();

    for column_row in column_rows {
        let (column_id, column_title) = column_row
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;
        
        let column_uuid = Uuid::parse_str(&column_id)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UUID: {}", e)))?;

        let mut tasks_stmt = conn.prepare(
            "SELECT id, title, description, due_date, priority, created_at, updated_at 
             FROM tasks WHERE column_id = ?1 ORDER BY position"
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

        let task_rows = tasks_stmt.query_map([&column_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        }).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

        let mut tasks = Vec::new();

        for task_row in task_rows {
            let (task_id, title, description, due_date, priority_str, created_at, updated_at) = task_row
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

            let task_uuid = Uuid::parse_str(&task_id)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UUID: {}", e)))?;

            let priority = match priority_str.as_str() {
                "Low" => Priority::Low,
                "Medium" => Priority::Medium,
                "High" => Priority::High,
                "Critical" => Priority::Critical,
                _ => Priority::Medium,
            };

            let due_date_parsed = if let Some(due_str) = due_date {
                Some(DateTime::parse_from_rfc3339(&due_str)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid date: {}", e)))?
                    .with_timezone(&Utc))
            } else {
                None
            };

            let created_at_parsed = DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid date: {}", e)))?
                .with_timezone(&Utc);

            let updated_at_parsed = DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid date: {}", e)))?
                .with_timezone(&Utc);

            let task = Task {
                id: task_uuid,
                title,
                description,
                due_date: due_date_parsed,
                priority,
                created_at: created_at_parsed,
                updated_at: updated_at_parsed,
            };

            tasks.push(task);
        }

        let column = Column {
            id: column_uuid,
            title: column_title,
            tasks,
        };

        columns.push(column);
    }

    Ok(Board {
        id: board_uuid,
        title: board_title,
        columns,
    })
}

