use clap::Parser;

use ccgen::input::cli::CliArgs;
use ccgen::input::config;
use ccgen::core::merger;
use ccgen::run;
use ccgen::types::RawConfig;

fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    let cfg_path = cli_args.config.clone();
    let cli_raw = cli_args.to_raw_config();

    let file_raw = if let Some(cfg_path) = &cfg_path {
        match config::parse(cfg_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: failed to parse config file: {}", e);
                RawConfig::default()
            }
        }
    } else {
        match config::find(&cli_raw.root) {
            Some(path) => config::parse(&path).unwrap_or_else(|e| {
                eprintln!("Warning: failed to parse config file: {}", e);
                RawConfig::default()
            }),
            None => RawConfig::default(),
        }
    };

    let merged = merger::merge(cli_raw, file_raw);
    run(merged)
}
