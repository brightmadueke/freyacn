pub mod toml;

use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(
    name = "freyacn",
    version = "0.1.0",
    author = "Bright Madueke",
    about = "Add freyacn components to your Freya project",
    long_about = "\
    freyacn-cli manages freyacn components in your Freya project.

Components are written into your project as source. There is no compiled
freyacn dependency to pull in — you add a component, the CLI writes its
files, and the code is yours from that point on. Edit it, extend it, or
delete it.

Typical workflow:

  $ freyacn init
  $ freyacn add button dialog card

Run `freyacn init` once per project. It creates `freyacn.toml`, sets up
theme tokens, and scaffolds the shared module that components depend on.
After that, `freyacn add` brings in components as you need them, and
`freyacn list` shows everything available.

Configuration lives in `freyacn.toml` at the project root. It records
where components are written, which registry to fetch from, and how the
theme is structured. Commit it alongside your code.

See https://freyacn.dev for documentation and the component registry.
"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        /// Project name (defaults to the directory name)
        #[arg(short, long)]
        name: Option<String>,

        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,

        /// Skip creating a git repository
        #[arg(long, default_value_t = true)]
        no_git: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            force,
            no_git,
        } => {
            initialize_project(name.as_deref(), force, no_git)
                .expect("Failed to initialize project");
        }
    }
}

fn initialize_project(
    name: Option<&str>,
    _force: bool,
    no_git: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // create directory if name exists, or create default directories
    match name {
        Some(name) => {
            // create the project directory
            let new_dir = std::env::current_dir()?.join(name);
            fs::create_dir_all(&new_dir)?;

            // create the components/ui directory
            fs::create_dir_all(&new_dir.join("components/ui"))?;

            // todo: check registry and create freyacn.toml
        }
        None => {}
    }

    // 5. Optional git init
    if !no_git {
        std::process::Command::new("git")
            .arg("init")
            .current_dir("")
            .status()?;
    }

    println!("Initialized");
    Ok(())
}
