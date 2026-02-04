//! Integration tests for taskctl CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cmd(data_dir: &str) -> Command {
    let mut cmd = Command::cargo_bin("taskctl").unwrap();
    cmd.args(["--data-dir", data_dir, "--no-color"]);
    cmd
}

fn setup() -> TempDir {
    TempDir::new().unwrap()
}

// ===== Task Lifecycle =====

#[test]
fn lifecycle_add_start_done() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d)
        .args(["add", "Test task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created task #1: Test task"));

    cmd(d)
        .args(["start", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Started task #1"));

    cmd(d)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed task #1"));

    // Verify task is done
    cmd(d)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn add_with_options() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d)
        .args([
            "add",
            "Complex task",
            "--due",
            "2025-12-31",
            "--tag",
            "backend,api",
            "--estimate",
            "3h",
            "--note",
            "Some notes",
        ])
        .assert()
        .success();

    cmd(d)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Complex task"))
        .stdout(predicate::str::contains("2025-12-31"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("api"))
        .stdout(predicate::str::contains("3h"))
        .stdout(predicate::str::contains("Some notes"));
}

// ===== Status Transitions =====

#[test]
fn idempotent_status_change() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Idem task"]).assert().success();
    cmd(d).args(["done", "1"]).assert().success();
    // Second done should succeed (idempotent)
    cmd(d)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed task #1"));
}

#[test]
fn pending_reopens_task() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Reopen task"]).assert().success();
    cmd(d).args(["done", "1"]).assert().success();
    cmd(d)
        .args(["pending", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened task #1"));
}

// ===== Dependency Flow =====

#[test]
fn dependency_flow() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Dep target"]).assert().success();
    cmd(d).args(["add", "Dependent"]).assert().success();
    cmd(d)
        .args(["depends", "2", "--on", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added dependency: #2 depends on #1",
        ));

    // Task 2 should be blocked
    cmd(d)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[blocked]"));

    // Complete task 1 should unblock task 2
    cmd(d)
        .args(["done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unblocked: #2"));

    // Task 2 should no longer be blocked
    cmd(d).args(["list"]).assert().success().stdout(
        predicate::str::contains("Dependent").and(predicate::str::contains("[blocked]").not()),
    );
}

#[test]
fn self_dependency_error() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Self dep"]).assert().success();
    cmd(d)
        .args(["depends", "1", "--on", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot depend on itself"));
}

#[test]
fn cyclic_dependency_error() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "A"]).assert().success();
    cmd(d).args(["add", "B"]).assert().success();
    cmd(d)
        .args(["depends", "2", "--on", "1"])
        .assert()
        .success();
    cmd(d)
        .args(["depends", "1", "--on", "2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cyclic dependency"));
}

// ===== Pin/Unpin =====

#[test]
fn pin_unpin() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Pin task"]).assert().success();
    cmd(d)
        .args(["pin", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned task #1"));

    cmd(d)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned:     Yes"));

    // Idempotent
    cmd(d).args(["pin", "1"]).assert().success();

    cmd(d)
        .args(["unpin", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unpinned task #1"));

    cmd(d)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned:     No"));
}

// ===== Edit =====

#[test]
fn edit_task() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Original"]).assert().success();
    cmd(d)
        .args(["edit", "1", "--title", "Updated", "--tag", "new-tag"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated task #1"));

    cmd(d)
        .args(["show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"))
        .stdout(predicate::str::contains("new-tag"));
}

// ===== Delete =====

#[test]
fn delete_task_with_force() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "To delete"]).assert().success();
    cmd(d)
        .args(["delete", "1", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted task #1"));

    cmd(d)
        .args(["show", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn delete_removes_dependency_refs() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Target"]).assert().success();
    cmd(d).args(["add", "Dependent"]).assert().success();
    cmd(d)
        .args(["depends", "2", "--on", "1"])
        .assert()
        .success();
    cmd(d).args(["delete", "1", "--force"]).assert().success();

    // Task 2 should no longer have dependencies
    cmd(d)
        .args(["show", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("depends on").not());
}

// ===== List / Next / Today =====

#[test]
fn list_hides_done_by_default() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Active"]).assert().success();
    cmd(d).args(["add", "Completed"]).assert().success();
    cmd(d).args(["done", "2"]).assert().success();

    // Default list should only show active task
    let output = cmd(d).args(["list"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Active"));
    assert!(!stdout.contains("Completed"));

    // --all shows both
    cmd(d)
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active"))
        .stdout(predicate::str::contains("Completed"));
}

#[test]
fn list_filter_by_tag() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d)
        .args(["add", "Backend", "--tag", "backend"])
        .assert()
        .success();
    cmd(d)
        .args(["add", "Frontend", "--tag", "frontend"])
        .assert()
        .success();

    let output = cmd(d).args(["list", "--tag", "backend"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Backend"));
    assert!(!stdout.contains("Frontend"));
}

#[test]
fn next_shows_top_task() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "A task"]).assert().success();
    cmd(d)
        .args(["next"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#1"));
}

#[test]
fn next_no_tasks() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d)
        .args(["next"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No actionable tasks found"));
}

// ===== Search =====

#[test]
fn search_by_title() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Authentication"]).assert().success();
    cmd(d).args(["add", "Database setup"]).assert().success();

    let output = cmd(d).args(["search", "auth"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Authentication"));
    assert!(!stdout.contains("Database"));
}

// ===== JSON Output =====

#[test]
fn json_output() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "JSON test"]).assert().success();

    let output = cmd(d).args(["list", "--json"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[test]
fn json_empty_list() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    let output = cmd(d).args(["list", "--json"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "[]");
}

// ===== Error Cases =====

#[test]
fn nonexistent_task_id() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d)
        .args(["show", "99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

// ===== Tree =====

#[test]
fn dependency_tree() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Root"]).assert().success();
    cmd(d).args(["add", "Child A"]).assert().success();
    cmd(d).args(["add", "Child B"]).assert().success();
    cmd(d)
        .args(["depends", "1", "--on", "2"])
        .assert()
        .success();
    cmd(d)
        .args(["depends", "1", "--on", "3"])
        .assert()
        .success();

    cmd(d)
        .args(["tree", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#1 Root"))
        .stdout(predicate::str::contains("#2 Child A"))
        .stdout(predicate::str::contains("#3 Child B"));
}

// ===== Undepends =====

#[test]
fn undepends() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "A"]).assert().success();
    cmd(d).args(["add", "B"]).assert().success();
    cmd(d)
        .args(["depends", "2", "--on", "1"])
        .assert()
        .success();
    cmd(d)
        .args(["undepends", "2", "--on", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no longer depends on"));
}

// ===== List with status filter =====

#[test]
fn list_by_status() {
    let dir = setup();
    let d = dir.path().to_str().unwrap();

    cmd(d).args(["add", "Active"]).assert().success();
    cmd(d).args(["add", "Done task"]).assert().success();
    cmd(d).args(["done", "2"]).assert().success();

    let output = cmd(d).args(["list", "--status", "done"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Done task"));
    assert!(!stdout.contains("Active"));
}
