use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, ErrorKind, Write},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

impl Task {
    pub fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title,
            completed: false,
        }
    }
}

pub fn add_task(path: impl AsRef<Path>, title: &str) -> Result<Task> {
    let title = title.trim();

    if title.is_empty() {
        return Err(anyhow!("title cannot be empty"));
    }

    let path = path.as_ref();
    let mut tasks = load_tasks(path)?;

    let new_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;

    tasks.push(Task::new(new_id, title.to_string()));

    save_tasks(path, &tasks)?;

    Ok(tasks.pop().expect("just pushed"))
}

pub fn remove_task(path: impl AsRef<Path>, id: u32) -> Result<Task> {
    let path = path.as_ref();
    let mut tasks = load_tasks(path)?;

    let Some(index) = tasks.iter().position(|task| task.id == id) else {
        return Err(anyhow!("task #{} does not exist", id));
    };

    let removed_task = tasks.remove(index);

    save_tasks(path, &tasks)?;

    Ok(removed_task)
}

pub fn complete_task(path: impl AsRef<Path>, id: u32) -> Result<Task> {
    let path = path.as_ref();
    let mut tasks = load_tasks(path)?;

    let Some(index) = tasks.iter().position(|task| task.id == id) else {
        return Err(anyhow!("task #{} does not exist", id));
    };

    tasks[index].completed = true;

    save_tasks(path, &tasks)?;

    Ok(tasks.remove(index))
}

pub fn load_tasks(path: &Path) -> Result<Vec<Task>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let reader = BufReader::new(file);

    let tasks = match serde_json::from_reader(reader) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => return Err(e.into()),
    };

    Ok(tasks)
}

pub fn save_tasks(path: &Path, tasks: &[Task]) -> Result<()> {
    let tmp_path = tmp_path(path);

    {
        let file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, tasks)?;
        writer.flush()?;
    }

    fs::rename(&tmp_path, path)?;

    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let mut path = std::env::temp_dir();
            path.push(format!("rustodo-test-{}-{id}.json", std::process::id()));
            let _ = std::fs::remove_file(&path);
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_file(tmp_path(&self.path));
        }
    }

    #[test]
    fn load_missing_file_is_empty() {
        let file = TempFile::new();
        assert_eq!(load_tasks(file.path()).unwrap(), Vec::new());
    }

    #[test]
    fn load_empty_file_is_empty() {
        let file = TempFile::new();
        File::create(file.path()).unwrap();
        assert_eq!(load_tasks(file.path()).unwrap(), Vec::new());
    }

    #[test]
    fn add_assigns_incrementing_ids() {
        let file = TempFile::new();

        let first = add_task(file.path(), "buy milk").unwrap();
        let second = add_task(file.path(), "write tests").unwrap();

        assert_eq!(first.id, 1);
        assert_eq!(second.id, 2);
        assert_eq!(load_tasks(file.path()).unwrap(), vec![first, second]);
    }

    #[test]
    fn add_trims_title() {
        let file = TempFile::new();
        let task = add_task(file.path(), "  buy milk  ").unwrap();
        assert_eq!(task.title, "buy milk");
    }

    #[test]
    fn add_rejects_blank_title() {
        let file = TempFile::new();
        assert!(add_task(file.path(), "   ").is_err());
    }

    #[test]
    fn remove_task_returns_and_persists() {
        let file = TempFile::new();
        let first = add_task(file.path(), "first").unwrap();
        let second = add_task(file.path(), "second").unwrap();

        let removed = remove_task(file.path(), first.id).unwrap();

        assert_eq!(removed, first);
        assert_eq!(load_tasks(file.path()).unwrap(), vec![second]);
    }

    #[test]
    fn remove_missing_task_is_error() {
        let file = TempFile::new();
        assert!(remove_task(file.path(), 99).is_err());
    }

    #[test]
    fn complete_task_marks_done_and_persists() {
        let file = TempFile::new();
        let task = add_task(file.path(), "buy milk").unwrap();

        let completed = complete_task(file.path(), task.id).unwrap();

        assert!(completed.completed);
        assert!(load_tasks(file.path()).unwrap()[0].completed);
    }

    #[test]
    fn complete_missing_task_is_error() {
        let file = TempFile::new();
        assert!(complete_task(file.path(), 99).is_err());
    }
}
