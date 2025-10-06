use clap::Parser;
use cocoatly_core::{
    types::{PackageArtifact, PackageName, Version, HashAlgorithm},
    config::Config,
    state::GlobalState,
};
use cocoatly_installer::install::{InstallContext, install_package};
use cocoatly_cli_bridge::output::{JsonOutput, OperationResult};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "cocoatly-install")]
#[command(about = "Install a package")]
struct Args {
    #[arg(long)]
    config: String,

    #[arg(long)]
    artifact_json: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let result = run(args).await;

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

async fn run(args: Args) -> anyhow::Result<OperationResult> {
    let config = Config::load_from_file(&args.config)?;
    let state = GlobalState::load_from_file(&config.storage.state_file)?;

    let artifact: PackageArtifact = serde_json::from_str(&args.artifact_json)?;

    let context = InstallContext::new(config, state)?;

    let installed = install_package(
        context,
        &artifact,
        vec![],
    ).await?;

    Ok(OperationResult {
        operation: "install".to_string(),
        package: installed.name.as_str().to_string(),
        version: installed.version.to_string(),
        message: format!("Successfully installed {} {}", installed.name.as_str(), installed.version.to_string()),
    })
}
