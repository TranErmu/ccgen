pub mod core;
pub mod error;
pub mod input;
pub mod output;
pub mod types;

use crate::types::{CcgenConfig, CompileEntry};

pub fn run(config: CcgenConfig) -> anyhow::Result<()> {
    let sources = core::discover::find_sources(&config);

    if sources.is_empty() {
        eprintln!("Warning: no source files found");
    }

    if config.verbose {
        eprintln!("[discover] found {} source files", sources.len());
    }

    let include_dirs = core::include_path::resolve_all(&config);

    if config.verbose {
        eprintln!("[include] resolved {} include directories", include_dirs.len());
    }

    let entries: Vec<CompileEntry> = sources
        .iter()
        .map(|source| core::compile_cmd::build_entry(&config, source, &include_dirs))
        .collect();

    if config.dry_run {
        output::writer::print_json(&entries)?;
    } else {
        output::writer::write_to_json(&entries, &config.output)?;
        if config.verbose {
            eprintln!("[output] wrote {} entries to {}", entries.len(), config.output.display());
        }
    }

    Ok(())
}
