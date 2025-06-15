use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub text: String,
    pub completed: bool,
    pub created_at: NaiveDate,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        Self::new_with_path(db_path)
    }

    pub fn new_with_path<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        let mut db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "threeaday")
            .context("Failed to get project directories")?;
        Ok(proj_dirs.data_dir().join("tasks.db"))
    }

    fn init_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                completed BOOLEAN DEFAULT FALSE,
                created_at DATE DEFAULT CURRENT_DATE
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
        
        let task_iter = stmt.query_map([today], |row| {
            Ok(Task {
                id: row.get(0)?,
                text: row.get(1)?,
                completed: row.get(2)?,
                created_at: row.get(3)?,
            })
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
    use tempfile::NamedTempFile;
    use chrono::Utc;

    fn create_test_db() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new_with_path(temp_file.path()).unwrap();
        (db, temp_file)
    }

    #[test]
    fn test_add_task() {
        let (mut db, _temp) = create_test_db();
        
        let task_id = db.add_task("test task").unwrap();
        assert!(task_id > 0);
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].text, "test task");
        assert!(!tasks[0].completed);
        assert_eq!(tasks[0].created_at, Utc::now().date_naive());
    }

    #[test]
    fn test_complete_task() {
        let (mut db, _temp) = create_test_db();
        
        let task_id = db.add_task("test task").unwrap();
        let success = db.complete_task(task_id).unwrap();
        assert!(success);
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].completed);
        
        // Completing again should return false
        let success_again = db.complete_task(task_id).unwrap();
        assert!(!success_again);
    }

    #[test]
    fn test_complete_nonexistent_task() {
        let (mut db, _temp) = create_test_db();
        
        let success = db.complete_task(999).unwrap();
        assert!(!success);
    }

    #[test]
    fn test_get_today_summary() {
        let (mut db, _temp) = create_test_db();
        
        // Empty state
        let (completed, total) = db.get_today_summary().unwrap();
        assert_eq!(completed, 0);
        assert_eq!(total, 0);
        
        // Add tasks
        let id1 = db.add_task("task 1").unwrap();
        let _id2 = db.add_task("task 2").unwrap();
        let _id3 = db.add_task("task 3").unwrap();
        
        let (completed, total) = db.get_today_summary().unwrap();
        assert_eq!(completed, 0);
        assert_eq!(total, 3);
        
        // Complete one task
        db.complete_task(id1).unwrap();
        let (completed, total) = db.get_today_summary().unwrap();
        assert_eq!(completed, 1);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_multiple_tasks_ordering() {
        let (mut db, _temp) = create_test_db();
        
        db.add_task("first").unwrap();
        db.add_task("second").unwrap();
        db.add_task("third").unwrap();
        
        let tasks = db.get_today_tasks().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].text, "first");
        assert_eq!(tasks[1].text, "second");
        assert_eq!(tasks[2].text, "third");
    }
}