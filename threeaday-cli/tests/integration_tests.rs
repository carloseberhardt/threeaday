use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("XDG_DATA_HOME", temp_dir.path());
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    temp_dir
}

fn threeaday_cmd() -> Command {
    Command::cargo_bin("threeaday").unwrap()
}

#[test]
fn test_add_task_command() {
    let _temp_dir = setup_test_env();
    
    threeaday_cmd()
        .arg("add")
        .arg("Test task")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added task Test task"));
}

#[test]
fn test_list_empty_tasks() {
    let _temp_dir = setup_test_env();
    
    threeaday_cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks for today yet"));
}

#[test]
fn test_list_with_tasks() {
    let _temp_dir = setup_test_env();
    
    // Add a task first
    threeaday_cmd()
        .arg("add")
        .arg("Test task")
        .assert()
        .success();
    
    // List tasks
    threeaday_cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Today's tasks:"))
        .stdout(predicate::str::contains("Test task"));
}

#[test]
fn test_complete_task() {
    let _temp_dir = setup_test_env();
    
    // Add a task first
    threeaday_cmd()
        .arg("add")
        .arg("Test task")
        .assert()
        .success();
    
    // Complete the task (assuming it gets ID 1)
    threeaday_cmd()
        .arg("done")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1 completed"));
}

#[test]
fn test_complete_nonexistent_task() {
    let _temp_dir = setup_test_env();
    
    threeaday_cmd()
        .arg("done")
        .arg("999")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 999 not found"));
}

#[test]
fn test_status_command() {
    let _temp_dir = setup_test_env();
    
    threeaday_cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Today's progress"));
}

#[test]
fn test_full_workflow() {
    let _temp_dir = setup_test_env();
    
    // Add multiple tasks
    threeaday_cmd()
        .arg("add")
        .arg("Task 1")
        .assert()
        .success();
        
    threeaday_cmd()
        .arg("add")
        .arg("Task 2")
        .assert()
        .success();
        
    threeaday_cmd()
        .arg("add")
        .arg("Task 3")
        .assert()
        .success();
    
    // Check status
    threeaday_cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("0/3 tasks completed"));
    
    // Complete one task
    threeaday_cmd()
        .arg("done")
        .arg("1")
        .assert()
        .success();
    
    // Check status again
    threeaday_cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1/3 tasks completed"));
        
    // Complete remaining tasks
    threeaday_cmd()
        .arg("done")
        .arg("2")
        .assert()
        .success();
        
    threeaday_cmd()
        .arg("done")
        .arg("3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daily goal achieved"));
}