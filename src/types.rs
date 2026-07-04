use std::path::PathBuf;
use serde::Serialize;

#[derive(Default)]
pub struct RawConfig {
    pub compiler: Option<String>,
    pub std: Option<String>,
    pub defines: Vec<String>,
    pub undefs: Vec<String>,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub no_gitignore: bool,
    pub root: PathBuf,
    pub output: Option<PathBuf>,
    pub verbose: bool,
    pub dry_run: bool,
}

pub struct CcgenConfig {
    pub root: PathBuf,
    pub compiler: Option<String>,
    pub std: Option<String>,
    pub defines: Vec<MacroDef>,
    pub undefs: Vec<String>,
    pub include_dirs: Vec<PathBuf>,
    pub include_exclude_dirs: Vec<PathBuf>,
    pub source_excludes: Vec<String>,
    pub no_gitignore: bool,
    pub output: PathBuf,
    pub verbose: bool,
    pub dry_run: bool,
}

pub struct MacroDef {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Serialize)]
pub struct CompileEntry {
    pub directory: PathBuf,
    pub file: PathBuf,
    pub arguments: Vec<String>,
}

pub enum ConfigSource {
    Cli,
    ConfigFile,
}
