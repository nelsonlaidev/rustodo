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
