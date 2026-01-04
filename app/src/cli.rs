use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ttt",
    about = "Track task time from the command line",
    after_help = "Examples:\n  ttt start \"Write docs\"\n  ttt pause\n  ttt resume\n  ttt status\n  ttt report\n  ttt stop\n  ttt location\n  ttt edit"
)]
pub struct Cli {
    #[arg(
        long = "data-file",
        value_name = "PATH",
        help = "Override the default data file location"
    )]
    pub data_file: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Start tracking a task")]
    Start {
        #[arg(value_name = "TASK", help = "Task name to track")]
        task: String,
    },
    #[command(about = "Stop the active or paused task")]
    Stop,
    #[command(about = "Pause the active task")]
    Pause,
    #[command(about = "Resume the paused task")]
    Resume,
    #[command(about = "Show the current task and elapsed time")]
    Status,
    #[command(about = "Show the data file location")]
    Location,
    #[command(about = "Show today's totals (default)")]
    Report {
        #[arg(long, help = "Report today's totals (default)")]
        today: bool,
    },
    #[command(about = "Edit a task name or time segments")]
    Edit {
        #[arg(long, value_name = "ID", help = "Task id to edit")]
        id: Option<String>,
        #[arg(
            long,
            value_name = "INDEX",
            help = "Task index from the list (1-based)"
        )]
        index: Option<usize>,
        #[arg(long, value_name = "NAME", help = "Rename the task")]
        name: Option<String>,
        #[arg(
            long,
            value_name = "RFC3339|now",
            help = "Override created time (RFC3339 or 'now')"
        )]
        created_at: Option<String>,
        #[arg(
            long,
            value_name = "RFC3339|open",
            help = "Override closed time (RFC3339 or 'open')"
        )]
        closed_at: Option<String>,
        #[arg(
            long,
            value_name = "INDEX,START,END",
            help = "Edit a segment (1-based). END can be 'open'."
        )]
        segment_edit: Vec<String>,
    },
}
