use std::process::exit;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rustodo::{add_task, complete_task, load_tasks, remove_task};

const DATA_FILE: &str = "data.json";

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
            let task = add_task(DATA_FILE, &title)?;

            println!("Added task #{}: {}", task.id, task.title);
        }
        Commands::List => {
            let tasks = load_tasks(DATA_FILE.as_ref())?;

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
            let task = remove_task(DATA_FILE, id)?;

            println!("Removed task #{}: {}", task.id, task.title);
        }
        Commands::Done { id } => {
            let task = complete_task(DATA_FILE, id)?;

            println!("Completed task #{}: {}", task.id, task.title);
        }
    }

    Ok(())
}
