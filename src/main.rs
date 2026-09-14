use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, ErrorKind, Write},
    process::exit,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "rustodo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { title: String },
    List,
    Remove { id: u32 },
    Done { id: u32 },
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    completed: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Self {
        Self {
            id,
            title,
            completed: false,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = handle_command(cli.command) {
        eprintln!("Error: {:#}", err);

        exit(1);
    }
}

fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Add { title } => {
            if title.trim().is_empty() {
                return Err(anyhow!("title cannot be empty"));
            }

            let title = title.trim().to_string();

            let mut tasks = load_tasks()?;

            let new_id = tasks.iter().map(|task| task.id).max().unwrap_or(0) + 1;

            let new_task = Task::new(new_id, title);

            tasks.push(new_task);

            save_tasks(&tasks)?;

            let added_task = tasks.last().unwrap();

            println!("Added task #{}: {}", added_task.id, added_task.title);
        }
        Commands::List => {
            let tasks = load_tasks()?;

            if tasks.is_empty() {
                println!("No tasks yet.");

                return Ok(());
            }

            for task in tasks {
                println!(
                    "{} [{}] {}",
                    task.id,
                    if task.completed { "x" } else { " " },
                    task.title
                )
            }
        }
        Commands::Remove { id } => {
            let mut tasks = load_tasks()?;

            let Some(index) = tasks.iter().position(|t| t.id == id) else {
                return Err(anyhow!("task #{} does not exist", id));
            };

            let removed_task = tasks.remove(index);

            save_tasks(&tasks)?;

            println!("Removed task #{}: {}", removed_task.id, removed_task.title);
        }
        Commands::Done { id } => {
            let mut tasks = load_tasks()?;

            let Some(index) = tasks.iter().position(|t| t.id == id) else {
                return Err(anyhow!("task #{} does not exist", id));
            };

            tasks[index].completed = true;

            save_tasks(&tasks)?;
            println!(
                "Completed task #{}: {}",
                tasks[index].id, tasks[index].title
            );
        }
    }

    Ok(())
}

fn save_tasks(tasks: &[Task]) -> Result<()> {
    let path = "data.json";
    let tmp_path = "data.json.tmp";

    {
        let file = File::create(tmp_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, tasks)?;
        writer.flush()?;
    }

    fs::rename(tmp_path, path)?;

    Ok(())
}

fn load_tasks() -> Result<Vec<Task>> {
    let file = match File::open("data.json") {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let reader = BufReader::new(file);

    let tasks: Vec<Task> = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => return Err(e.into()),
    };

    Ok(tasks)
}
