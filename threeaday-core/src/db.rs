use crate::task::Task;
use crate::utils::get_data_dir;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {}", parent.display()))?;
        }
        
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {}", db_path.display()))?;
        
        let mut db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }
    
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = get_data_dir()?;
        Ok(data_dir.join("tasks.db"))
    }
    
    fn init_tables(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATE NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_task(&mut self, text: &str) -> Result<i64> {
        let today = Utc::now().date_naive();
        let _rows_affected = self.conn.execute(
            "INSERT INTO tasks (text, created_at) VALUES (?1, ?2)",
            params![text, today],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_today_tasks(&self) -> Result<Vec<Task>> {
        let today = Utc::now().date_naive();
        let mut stmt = self.conn.prepare(
            "SELECT id, text, completed, created_at FROM tasks WHERE created_at = ?1 ORDER BY id"
        )?;
        
        let task_iter = stmt.query_map(params![today], |row| {
            Ok(Task::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        
        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn complete_task(&mut self, id: i64) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "UPDATE tasks SET completed = TRUE WHERE id = ?1 AND completed = FALSE",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn get_today_summary(&self) -> Result<(usize, usize)> {
        let tasks = self.get_today_tasks()?;
        let completed = tasks.iter().filter(|t| t.completed).count();
        Ok((completed, tasks.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let _data_dir = temp_dir.path().join("threeaday");
        
        // Set environment variable to use test database
        env::set_var("XDG_DATA_HOME", temp_dir.path());
        
        let db = Database::new().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_add_task() {
        let (mut db, _temp_dir) = setup_test_db();
        
        let task_id = db.add_task("Test task").unwrap();
        assert!(task_id > 0);
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].text, "Test task");
        assert!(!tasks[0].completed);
    }

    #[test]
    fn test_complete_task() {
        let (mut db, _temp_dir) = setup_test_db();
        
        let task_id = db.add_task("Test task").unwrap();
        let success = db.complete_task(task_id).unwrap();
        assert!(success);
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].completed);
    }

    #[test]
    fn test_complete_nonexistent_task() {
        let (mut db, _temp_dir) = setup_test_db();
        
        let success = db.complete_task(999).unwrap();
        assert!(!success);
    }

    #[test]
    fn test_get_today_summary() {
        let (mut db, _temp_dir) = setup_test_db();
        
        db.add_task("Task 1").unwrap();
        let task2_id = db.add_task("Task 2").unwrap();
        db.add_task("Task 3").unwrap();
        
        db.complete_task(task2_id).unwrap();
        
        let (completed, total) = db.get_today_summary().unwrap();
        assert_eq!(completed, 1);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_multiple_tasks_ordering() {
        let (mut db, _temp_dir) = setup_test_db();
        
        db.add_task("First task").unwrap();
        db.add_task("Second task").unwrap();
        db.add_task("Third task").unwrap();
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].text, "First task");
        assert_eq!(tasks[1].text, "Second task");
        assert_eq!(tasks[2].text, "Third task");
    }
}