# rustodo

A tiny command-line todo list written in Rust. Tasks are stored locally in a
`data.json` file next to where you run the command.

This is a learning project, so it is intentionally small: one binary, one file,
a handful of subcommands.

## Requirements

- Rust 1.85 or newer (the project uses the 2024 edition)

## Build

```sh
cargo build --release
```

The binary is written to `target/release/rustodo`.

## Usage

Add a task:

```sh
rustodo add "buy milk"
```

List all tasks:

```sh
rustodo list
```

Output looks like:

```
1 [ ] buy milk
2 [x] write README
```

Mark a task as done:

```sh
rustodo done 1
```

Remove a task:

```sh
rustodo remove 1
```

## Commands

| Command       | Description                    |
| ------------- | ------------------------------ |
| `add <title>` | Add a new task                 |
| `list`        | Show every task                |
| `done <id>`   | Mark the task with `<id>` done |
| `remove <id>` | Delete the task with `<id>`    |

Run `rustodo --help` for the built-in help.

## Storage

Tasks live in `data.json` in the current working directory. The file is created
on the first `add`. If it does not exist, the list is simply treated as empty.
The file is ignored by git, so your personal tasks stay local.

## Project layout

```
src/main.rs   CLI definition, commands, and JSON read/write
Cargo.toml    Package metadata and dependencies
```

## Dependencies

- [clap](https://crates.io/crates/clap) — command-line argument parsing
- [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) — JSON serialization
- [anyhow](https://crates.io/crates/anyhow) — error handling
