use clap::Parser;
use cocoatly_core::{
    types::PackageName,
    config::Config,
    state::GlobalState,
};
use cocoatly_installer::uninstall::uninstall_package;
use cocoatly_cli_bridge::output::{JsonOutput, OperationResult};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "cocoatly-uninstall")]
#[command(about = "Uninstall a package")]
struct Args {
    #[arg(long)]
    config: String,

    #[arg(long)]
    package: String,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let result = run(args);

    match result {
        Ok(op_result) => {
            JsonOutput::success(op_result).print();
            std::process::exit(0);
        }
        Err(e) => {
            JsonOutput::<()>::failure(e.to_string()).print();
            std::process::exit(1);
        }
    }
}

fn run(args: Args) -> anyhow::Result<OperationResult> {
    let config = Config::load_from_file(&args.config)?;
    let state = GlobalState::load_from_file(&config.storage.state_file)?;

    let package_name = PackageName::new(args.package.clone());

    let version = state
        .get_package(&package_name)
        .map(|p| p.version.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    uninstall_package(config, state, &package_name)?;

    Ok(OperationResult {
        operation: "uninstall".to_string(),
        package: args.package,
        version,
        message: "Successfully uninstalled package".to_string(),
    })
}
