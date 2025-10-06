use clap::Parser;
use cocoatly_core::{
    types::PackageName,
    config::Config,
    state::GlobalState,
};
use cocoatly_installer::verify::verify_installation;
use cocoatly_cli_bridge::output::JsonOutput;
use serde::{Serialize, Deserialize};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "cocoatly-verify")]
#[command(about = "Verify package installation")]
struct Args {
    #[arg(long)]
    config: String,

    #[arg(long)]
    package: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerificationOutput {
    package: String,
    valid: bool,
    missing_files: Vec<String>,
    corrupted_files: Vec<String>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let result = run(args);

    match result {
        Ok(verification) => {
            JsonOutput::success(verification).print();
            std::process::exit(if verification.valid { 0 } else { 1 });
        }
        Err(e) => {
            JsonOutput::<()>::failure(e.to_string()).print();
            std::process::exit(1);
        }
    }
}

fn run(args: Args) -> anyhow::Result<VerificationOutput> {
    let config = Config::load_from_file(&args.config)?;
    let state = GlobalState::load_from_file(&config.storage.state_file)?;

    let package_name = PackageName::new(args.package.clone());

    let result = verify_installation(&config, &state, &package_name)?;

    Ok(VerificationOutput {
        package: args.package,
        valid: result.valid,
        missing_files: result.missing_files,
        corrupted_files: result.corrupted_files,
    })
}
