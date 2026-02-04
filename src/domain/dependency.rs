//! Dependency management with cycle detection.

use crate::domain::status::Status;
use crate::domain::task::Task;
use crate::error::TaskCtlError;
use std::collections::HashSet;

/// Add a dependency, checking for self-reference and cycles.
pub fn add_dependency(
    task_id: u32,
    depends_on_id: u32,
    all_tasks: &[Task],
) -> Result<(), TaskCtlError> {
    if task_id == depends_on_id {
        return Err(TaskCtlError::SelfDependency(task_id));
    }

    // Check the target exists
    if !all_tasks.iter().any(|t| t.id == depends_on_id) {
        return Err(TaskCtlError::TaskNotFound(depends_on_id));
    }

    // Check for cycles: if we add task_id -> depends_on_id,
    // then from depends_on_id we should NOT be able to reach task_id.
    if would_create_cycle(task_id, depends_on_id, all_tasks) {
        let cycle_path = find_cycle_path(task_id, depends_on_id, all_tasks);
        return Err(TaskCtlError::CyclicDependency(cycle_path));
    }

    Ok(())
}

/// Check if adding task_id -> depends_on_id would create a cycle.
fn would_create_cycle(task_id: u32, depends_on_id: u32, all_tasks: &[Task]) -> bool {
    // DFS from depends_on_id, see if we can reach task_id
    let mut visited = HashSet::new();
    let mut stack = vec![depends_on_id];

    while let Some(current) = stack.pop() {
        if current == task_id {
            return true;
        }
        if visited.insert(current) {
            if let Some(task) = all_tasks.iter().find(|t| t.id == current) {
                for &dep in &task.depends_on {
                    stack.push(dep);
                }
            }
        }
    }
    false
}

fn find_cycle_path(task_id: u32, depends_on_id: u32, all_tasks: &[Task]) -> String {
    // Simple path representation
    let mut path = vec![format!("#{task_id}")];
    path.push(format!("#{depends_on_id}"));

    let mut current = depends_on_id;
    let mut visited = HashSet::new();
    visited.insert(task_id);

    while current != task_id {
        visited.insert(current);
        if let Some(task) = all_tasks.iter().find(|t| t.id == current) {
            if let Some(&next) = task
                .depends_on
                .iter()
                .find(|&&dep| dep == task_id || !visited.contains(&dep))
            {
                path.push(format!("#{next}"));
                current = next;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    path.join(" -> ")
}

/// Remove a dependency from task's depends_on list.
pub fn remove_dependency(task: &mut Task, depends_on_id: u32) {
    task.depends_on.retain(|&id| id != depends_on_id);
}

/// Check if a task is blocked (has incomplete dependencies).
pub fn is_blocked(task: &Task, all_tasks: &[Task]) -> bool {
    task.depends_on.iter().any(|&dep_id| {
        all_tasks
            .iter()
            .find(|t| t.id == dep_id)
            .is_some_and(|t| t.status != Status::Done)
    })
}

/// Get IDs of tasks that this task is blocking (tasks that depend on this one).
pub fn get_blocking_tasks(task_id: u32, all_tasks: &[Task]) -> Vec<u32> {
    all_tasks
        .iter()
        .filter(|t| t.depends_on.contains(&task_id) && t.status != Status::Done)
        .map(|t| t.id)
        .collect()
}

/// Build a dependency tree for display.
#[derive(Debug)]
pub struct TreeNode {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub children: Vec<TreeNode>,
}

pub fn get_dependency_tree(task_id: u32, all_tasks: &[Task]) -> Option<TreeNode> {
    let task = all_tasks.iter().find(|t| t.id == task_id)?;
    let mut visited = HashSet::new();
    Some(build_tree(task, all_tasks, &mut visited))
}

fn build_tree(task: &Task, all_tasks: &[Task], visited: &mut HashSet<u32>) -> TreeNode {
    visited.insert(task.id);
    let children = task
        .depends_on
        .iter()
        .filter_map(|&dep_id| {
            if visited.contains(&dep_id) {
                return None;
            }
            all_tasks
                .iter()
                .find(|t| t.id == dep_id)
                .map(|dep_task| build_tree(dep_task, all_tasks, visited))
        })
        .collect();

    TreeNode {
        id: task.id,
        title: task.title.clone(),
        status: task.status,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;

    fn make_task(id: u32, deps: Vec<u32>) -> Task {
        let mut t = Task::new(id, format!("Task {id}"));
        t.depends_on = deps;
        t
    }

    fn make_done_task(id: u32) -> Task {
        let mut t = Task::new(id, format!("Task {id}"));
        t.status = Status::Done;
        t
    }

    #[test]
    fn add_valid_dependency() {
        let tasks = vec![make_task(1, vec![]), make_task(2, vec![])];
        assert!(add_dependency(2, 1, &tasks).is_ok());
    }

    #[test]
    fn self_dependency() {
        let tasks = vec![make_task(1, vec![])];
        assert!(matches!(
            add_dependency(1, 1, &tasks),
            Err(TaskCtlError::SelfDependency(1))
        ));
    }

    #[test]
    fn target_not_found() {
        let tasks = vec![make_task(1, vec![])];
        assert!(matches!(
            add_dependency(1, 99, &tasks),
            Err(TaskCtlError::TaskNotFound(99))
        ));
    }

    #[test]
    fn direct_cycle() {
        // 1 depends on 2, trying to add 2 depends on 1
        let tasks = vec![make_task(1, vec![2]), make_task(2, vec![])];
        assert!(add_dependency(2, 1, &tasks).is_err());
    }

    #[test]
    fn indirect_cycle() {
        // 1->2->3, trying to add 3->1
        let tasks = vec![
            make_task(1, vec![2]),
            make_task(2, vec![3]),
            make_task(3, vec![]),
        ];
        assert!(add_dependency(3, 1, &tasks).is_err());
    }

    #[test]
    fn no_cycle_in_dag() {
        // Diamond: 1->2, 1->3, 2->4, 3->4
        let tasks = vec![
            make_task(1, vec![2, 3]),
            make_task(2, vec![4]),
            make_task(3, vec![4]),
            make_task(4, vec![]),
        ];
        // Adding 1->4 is fine (no cycle)
        assert!(add_dependency(1, 4, &tasks).is_ok());
    }

    #[test]
    fn is_blocked_with_pending_dep() {
        let tasks = vec![make_task(1, vec![]), make_task(2, vec![1])];
        assert!(is_blocked(&tasks[1], &tasks));
    }

    #[test]
    fn is_not_blocked_with_done_dep() {
        let tasks = vec![make_done_task(1), make_task(2, vec![1])];
        assert!(!is_blocked(&tasks[1], &tasks));
    }

    #[test]
    fn is_not_blocked_no_deps() {
        let tasks = vec![make_task(1, vec![])];
        assert!(!is_blocked(&tasks[0], &tasks));
    }

    #[test]
    fn blocking_tasks() {
        let tasks = vec![
            make_task(1, vec![]),
            make_task(2, vec![1]),
            make_task(3, vec![1]),
            make_done_task(4),
        ];
        let blocking = get_blocking_tasks(1, &tasks);
        assert_eq!(blocking.len(), 2);
        assert!(blocking.contains(&2));
        assert!(blocking.contains(&3));
    }

    #[test]
    fn dependency_tree() {
        let tasks = vec![
            make_task(1, vec![2, 3]),
            make_task(2, vec![]),
            make_task(3, vec![]),
        ];
        let tree = get_dependency_tree(1, &tasks).unwrap();
        assert_eq!(tree.id, 1);
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn remove_dep() {
        let mut task = make_task(1, vec![2, 3]);
        remove_dependency(&mut task, 2);
        assert_eq!(task.depends_on, vec![3]);
    }
}
