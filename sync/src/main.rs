use clap::{Parser, Subcommand};
use omnity_indexer_sync::{tasks::execute_sync_tasks, utils::*};

use std::env;

// #[cfg(debug_assertions)]
// use dotenvy::dotenv;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// the path to the config file
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// start sync tasks
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(debug_assertions)]
    // dotenv().ok();

    let cli = Cli::parse();
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    // println!("exe_dir: {:?}", exe_dir);

    let default_config_filename = "config.toml";
    let default_config_path = exe_dir.join(default_config_filename);

    // If the user didn't specify a config, use the default
    let config_path = cli
        .config
        .unwrap_or_else(|| default_config_path.to_str().unwrap().to_string());

    let settings = Settings::new(&config_path)?;
    // println!("settings: {:?}", settings);
    set_config(settings);
    // init log
    let log_config = read_config(|c| c.log_config.to_owned());
    if let Err(e) = log4rs::init_file(log_config, Default::default()) {
        eprintln!("init log failed: {}", e);
        std::process::exit(1);
    }

    match &cli.command {
        Some(Commands::Start) => {
            // init database
            let db_url = read_config(|c| c.database_url.to_owned());
            let db = Database::new(db_url).await;

            execute_sync_tasks(db.get_connection()).await;
        }
        None => {}
    }

    Ok(())
}
