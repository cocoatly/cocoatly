use clap::{Parser, Subcommand};
use cocoatly_core::{
    config::Config,
    state::GlobalState,
};
use cocoatly_cli_bridge::output::JsonOutput;
use serde::{Serialize, Deserialize};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "cocoatly-state")]
#[command(about = "Manage package state")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Read {
        #[arg(long)]
        config: String,
    },
    Write {
        #[arg(long)]
        config: String,
        #[arg(long)]
        state_json: String,
    },
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let result = run(args);

    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(e) => {
            JsonOutput::<()>::failure(e.to_string()).print();
            std::process::exit(1);
        }
    }
}

fn run(args: Args) -> anyhow::Result<String> {
    match args.command {
        Commands::Read { config } => {
            let config = Config::load_from_file(&config)?;
            let state = GlobalState::load_from_file(&config.storage.state_file)?;
            let json = serde_json::to_string_pretty(&state)?;
            Ok(json)
        }
        Commands::Write { config, state_json } => {
            let config = Config::load_from_file(&config)?;
            let state: GlobalState = serde_json::from_str(&state_json)?;
            state.save_to_file(&config.storage.state_file)?;
            Ok("State saved successfully".to_string())
        }
    }
}
