use std::path::{Path, PathBuf};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::AppHandle;
use crate::control::task_manager::{TASK_MANAGER, TaskStatus};

static BACKUP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Checks if task is paused and blocks, or errors if cancelled
pub async fn pause_point(task_id: &str) -> Result<(), String> {
    if TASK_MANAGER.is_cancelled(task_id) {
        return Err("Task was cancelled".to_string());
    }
    TASK_MANAGER.wait_if_paused(task_id).await;
    if TASK_MANAGER.is_cancelled(task_id) {
        return Err("Task was cancelled".to_string());
    }
    Ok(())
}

/// Perform a transaction-safe file copy. Registers rollback to delete the copy.
pub async fn safe_copy<P: AsRef<Path>, Q: AsRef<Path>>(
    task_id: &str,
    from: P,
    to: Q,
) -> Result<u64, String> {
    pause_point(task_id).await?;

    let from_path = from.as_ref();
    let to_path = to.as_ref();

    if !from_path.exists() {
        return Err(format!("Source path does not exist: {:?}", from_path));
    }

    // Perform copy
    let bytes = fs::copy(from_path, to_path).map_err(|e| e.to_string())?;

    // Track affected file
    TASK_MANAGER.register_affected_file(task_id, &to_path.to_string_lossy());

    // Register rollback: delete target file
    let to_clone = to_path.to_path_buf();
    TASK_MANAGER.register_rollback_action(
        task_id,
        &format!("Delete copied file {:?}", to_clone),
        move || {
            if to_clone.exists() {
                fs::remove_file(&to_clone).map_err(|e| e.to_string())?;
                println!("Rollback: Successfully removed copied file {:?}", to_clone);
            }
            Ok(())
        },
    );

    Ok(bytes)
}

/// Perform a transaction-safe file rename/move. Registers rollback to move it back.
pub async fn safe_rename<P: AsRef<Path>, Q: AsRef<Path>>(
    task_id: &str,
    from: P,
    to: Q,
) -> Result<(), String> {
    pause_point(task_id).await?;

    let from_path = from.as_ref().to_path_buf();
    let to_path = to.as_ref().to_path_buf();

    if !from_path.exists() {
        return Err(format!("Source path does not exist: {:?}", from_path));
    }

    // Perform rename/move
    fs::rename(&from_path, &to_path).map_err(|e| e.to_string())?;

    // Track affected files
    TASK_MANAGER.register_affected_file(task_id, &from_path.to_string_lossy());
    TASK_MANAGER.register_affected_file(task_id, &to_path.to_string_lossy());

    // Register rollback: move it back
    let from_clone = from_path.clone();
    let to_clone = to_path.clone();
    TASK_MANAGER.register_rollback_action(
        task_id,
        &format!("Restore renamed file from {:?} back to {:?}", to_clone, from_clone),
        move || {
            if to_clone.exists() {
                fs::rename(&to_clone, &from_clone).map_err(|e| e.to_string())?;
                println!("Rollback: Successfully moved {:?} back to {:?}", to_clone, from_clone);
            }
            Ok(())
        },
    );

    Ok(())
}

/// Perform a transaction-safe file delete. Backs up the file to temp before deletion.
pub async fn safe_delete<P: AsRef<Path>>(
    task_id: &str,
    path: P,
) -> Result<(), String> {
    pause_point(task_id).await?;

    let path_ref = path.as_ref().to_path_buf();
    if !path_ref.exists() {
        return Ok(());
    }

    // Perform deletion with backup creation
    let count = BACKUP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_dir = std::env::temp_dir().join("alok_file_backups");
    let _ = fs::create_dir_all(&temp_dir);
    let backup_filename = format!("backup_file_{}_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(), count);
    let backup_path = temp_dir.join(backup_filename);

    // Copy to backup
    fs::copy(&path_ref, &backup_path).map_err(|e| e.to_string())?;

    // Perform delete
    fs::remove_file(&path_ref).map_err(|e| e.to_string())?;

    // Track affected file
    TASK_MANAGER.register_affected_file(task_id, &path_ref.to_string_lossy());

    // Register rollback: restore file from backup
    let path_clone = path_ref.clone();
    let backup_clone = backup_path.clone();
    TASK_MANAGER.register_rollback_action(
        task_id,
        &format!("Restore deleted file {:?}", path_clone),
        move || {
            if backup_clone.exists() {
                // Ensure parent directory exists
                if let Some(parent) = path_clone.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                fs::copy(&backup_clone, &path_clone).map_err(|e| e.to_string())?;
                let _ = fs::remove_file(&backup_clone);
                println!("Rollback: Successfully restored deleted file {:?}", path_clone);
            }
            Ok(())
        },
    );

    Ok(())
}

/// Trigger rollback on a task
pub fn rollback_task(task_id: &str) -> Result<(), String> {
    println!("TaskCancellation: Rolling back task '{}'...", task_id);
    let rollback_actions = TASK_MANAGER.take_rollback_actions(task_id);
    
    // Execute rollbacks in reverse order
    let mut failures = Vec::new();
    for action in rollback_actions.into_iter().rev() {
        println!("TaskCancellation: Running rollback: {}", action.description);
        if let Err(e) = (action.undo)() {
            eprintln!("TaskCancellation: Rollback step failed: {}", e);
            failures.push(format!("{}: {}", action.description, e));
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("Rollback encountered failures: {:?}", failures))
    }
}

/// Cancels and rolls back a task
pub fn cancel_and_rollback_task(task_id: &str) -> Result<(), String> {
    TASK_MANAGER.update_status(task_id, TaskStatus::Cancelled);
    rollback_task(task_id)
}
