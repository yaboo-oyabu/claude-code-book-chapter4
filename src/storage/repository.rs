//! Task persistence (CRUD operations).

use crate::domain::task::{Task, TaskWithNote};
use crate::error::TaskCtlError;
use crate::storage::lock::FileLock;
use crate::storage::markdown;
use crate::storage::meta::Meta;
use std::path::PathBuf;

pub struct Repository {
    data_dir: PathBuf,
}

impl Repository {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Ensure the data directory exists.
    pub fn ensure_dir(&self) -> Result<(), TaskCtlError> {
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)?;
        }
        Ok(())
    }

    fn task_path(&self, id: u32) -> PathBuf {
        self.data_dir.join(format!("{id}.md"))
    }

    /// Create a new task. Allocates an ID and writes the file.
    pub fn create(
        &self,
        title: String,
        mut builder: impl FnMut(&mut Task),
    ) -> Result<TaskWithNote, TaskCtlError> {
        self.ensure_dir()?;
        let _lock = FileLock::acquire(&self.data_dir)?;

        let mut meta = Meta::load(&self.data_dir)?;
        let id = meta.allocate_id();

        let mut task = Task::new(id, title);
        builder(&mut task);

        let content = markdown::serialize(&task, "")?;
        std::fs::write(self.task_path(id), content)?;
        meta.save(&self.data_dir)?;

        Ok(TaskWithNote {
            task,
            note: String::new(),
        })
    }

    /// Read a single task by ID.
    pub fn read(&self, id: u32) -> Result<TaskWithNote, TaskCtlError> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(TaskCtlError::TaskNotFound(id));
        }
        let content = std::fs::read_to_string(&path)?;
        let path_str = path.to_string_lossy().into_owned();
        let (task, note): (Task, String) = markdown::parse(&content, &path_str)?;
        Ok(TaskWithNote { task, note })
    }

    /// Read all tasks in the data directory.
    pub fn read_all(&self) -> Result<Vec<TaskWithNote>, TaskCtlError> {
        if !self.data_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let content = std::fs::read_to_string(&path)?;
                let path_str = path.to_string_lossy().into_owned();
                match markdown::parse::<Task>(&content, &path_str) {
                    Ok((task, note)) => tasks.push(TaskWithNote { task, note }),
                    Err(e) => {
                        eprintln!("Warning: skipping {}: {e}", path.display());
                    }
                }
            }
        }

        tasks.sort_by_key(|t| t.task.id);
        Ok(tasks)
    }

    /// Update an existing task.
    pub fn update(&self, task_with_note: &TaskWithNote) -> Result<(), TaskCtlError> {
        let path = self.task_path(task_with_note.task.id);
        if !path.exists() {
            return Err(TaskCtlError::TaskNotFound(task_with_note.task.id));
        }
        let _lock = FileLock::acquire(&self.data_dir)?;
        let content = markdown::serialize(&task_with_note.task, &task_with_note.note)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Delete a task and remove it from other tasks' depends_on lists.
    pub fn delete(&self, id: u32) -> Result<(), TaskCtlError> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(TaskCtlError::TaskNotFound(id));
        }
        let _lock = FileLock::acquire(&self.data_dir)?;
        std::fs::remove_file(path)?;

        // Remove references from other tasks' depends_on
        let all = self.read_all_unlocked()?;
        for mut tw in all {
            if tw.task.depends_on.contains(&id) {
                tw.task.depends_on.retain(|&dep_id| dep_id != id);
                let content = markdown::serialize(&tw.task, &tw.note)?;
                std::fs::write(self.task_path(tw.task.id), content)?;
            }
        }

        Ok(())
    }

    /// Read all tasks without acquiring a lock (for internal use when lock is already held).
    fn read_all_unlocked(&self) -> Result<Vec<TaskWithNote>, TaskCtlError> {
        if !self.data_dir.exists() {
            return Ok(Vec::new());
        }
        let mut tasks = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let content = std::fs::read_to_string(&path)?;
                let path_str = path.to_string_lossy().into_owned();
                if let Ok((task, note)) = markdown::parse::<Task>(&content, &path_str) {
                    tasks.push(TaskWithNote { task, note });
                }
            }
        }
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_repo() -> (tempfile::TempDir, Repository) {
        let dir = tempfile::TempDir::new().unwrap();
        let repo = Repository::new(dir.path().to_path_buf());
        (dir, repo)
    }

    #[test]
    fn create_and_read() {
        let (_dir, repo) = test_repo();
        let tw = repo.create("Test task".to_string(), |_| {}).unwrap();
        assert_eq!(tw.task.id, 1);
        assert_eq!(tw.task.title, "Test task");

        let read = repo.read(1).unwrap();
        assert_eq!(read.task.title, "Test task");
    }

    #[test]
    fn create_multiple() {
        let (_dir, repo) = test_repo();
        let t1 = repo.create("First".to_string(), |_| {}).unwrap();
        let t2 = repo.create("Second".to_string(), |_| {}).unwrap();
        assert_eq!(t1.task.id, 1);
        assert_eq!(t2.task.id, 2);
    }

    #[test]
    fn read_nonexistent() {
        let (_dir, repo) = test_repo();
        repo.ensure_dir().unwrap();
        assert!(repo.read(99).is_err());
    }

    #[test]
    fn read_all_empty() {
        let (_dir, repo) = test_repo();
        let tasks = repo.read_all().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn read_all_with_tasks() {
        let (_dir, repo) = test_repo();
        repo.create("A".to_string(), |_| {}).unwrap();
        repo.create("B".to_string(), |_| {}).unwrap();
        let tasks = repo.read_all().unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn update_task() {
        let (_dir, repo) = test_repo();
        let mut tw = repo.create("Original".to_string(), |_| {}).unwrap();
        tw.task.title = "Updated".to_string();
        repo.update(&tw).unwrap();
        let read = repo.read(1).unwrap();
        assert_eq!(read.task.title, "Updated");
    }

    #[test]
    fn delete_task() {
        let (_dir, repo) = test_repo();
        repo.create("To delete".to_string(), |_| {}).unwrap();
        repo.delete(1).unwrap();
        assert!(repo.read(1).is_err());
    }

    #[test]
    fn delete_removes_dependency_refs() {
        let (_dir, repo) = test_repo();
        repo.create("Dep target".to_string(), |_| {}).unwrap();
        repo.create("Dependent".to_string(), |t| {
            t.depends_on = vec![1];
        })
        .unwrap();

        repo.delete(1).unwrap();
        let t2 = repo.read(2).unwrap();
        assert!(t2.task.depends_on.is_empty());
    }
}
