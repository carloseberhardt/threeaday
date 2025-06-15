use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::env;
use std::sync::Mutex;

// Serialize test execution to avoid env var conflicts
static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn with_test_env<F>(test: F) 
where 
    F: FnOnce(&TempDir),
{
    let _guard = TEST_MUTEX.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let old_data_home = env::var("XDG_DATA_HOME").ok();
    
    env::set_var("XDG_DATA_HOME", temp_dir.path());
    
    test(&temp_dir);
    
    // Restore original env var
    match old_data_home {
        Some(path) => env::set_var("XDG_DATA_HOME", path),
        None => env::remove_var("XDG_DATA_HOME"),
    }
}

#[test]
fn test_add_task_command() {
    with_test_env(|_temp_dir| {
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("add")
            .arg("test task")
            .assert()
            .success()
            .stdout(predicate::str::contains("Added task: test task"));
    });
}

#[test]
fn test_list_empty_tasks() {
    with_test_env(|_temp_dir| {
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("No tasks for today"));
    });
}

#[test]
fn test_list_with_tasks() {
    with_test_env(|_temp_dir| {
        // Add a task first
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("add").arg("test task").assert().success();
        
        // List tasks
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("[ ] 1 - test task"));
    });
}

#[test]
fn test_complete_task() {
    with_test_env(|_temp_dir| {
        // Add a task
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("add").arg("test task").assert().success();
        
        // Complete the task
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("done")
            .arg("1")
            .assert()
            .success()
            .stdout(predicate::str::contains("Task 1 completed! 🎉"));
    });
}

#[test]
fn test_status_command() {
    with_test_env(|_temp_dir| {
        // Empty status
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("status")
            .assert()
            .success()
            .stdout(predicate::str::contains("Today's progress: 0/0"));
        
        // Add tasks and check status
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("add").arg("task 1").assert().success();
        
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("status")
            .assert()
            .success()
            .stdout(predicate::str::contains("Today's progress: 0/1"));
    });
}

#[test]
fn test_complete_nonexistent_task() {
    with_test_env(|_temp_dir| {
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("done")
            .arg("999")
            .assert()
            .success()
            .stdout(predicate::str::contains("Task 999 not found"));
    });
}

#[test]
fn test_full_workflow() {
    with_test_env(|_temp_dir| {
        // Add three tasks
        for i in 1..=3 {
            let mut cmd = Command::cargo_bin("threeaday").unwrap();
            cmd.arg("add")
                .arg(format!("task {}", i))
                .assert()
                .success();
        }
        
        // Complete all three
        for i in 1..=3 {
            let mut cmd = Command::cargo_bin("threeaday").unwrap();
            cmd.arg("done")
                .arg(i.to_string())
                .assert()
                .success();
        }
        
        // Check final status
        let mut cmd = Command::cargo_bin("threeaday").unwrap();
        cmd.arg("status")
            .assert()
            .success()
            .stdout(predicate::str::contains("🎯 Goal achieved!"));
    });
}