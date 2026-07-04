use std::path::PathBuf;
use serde::Serialize;

#[derive(Default)]
pub struct RawConfig {
    pub compiler: Option<String>,
    pub std_c: Option<String>,
    pub std_cpp: Option<String>,
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
    pub std_c: Option<String>,
    pub std_cpp: Option<String>,
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

pub fn parse_std(value: &str) -> (Option<String>, Option<String>) {
    let mut std_c = None;
    let mut std_cpp = None;

    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.contains("++") {
            std_cpp = Some(part.to_string());
        } else {
            std_c = Some(part.to_string());
        }
    }

    (std_c, std_cpp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_std_comma_separated() {
        let (c, cpp) = parse_std("c11,c++17");
        assert_eq!(c, Some("c11".to_string()));
        assert_eq!(cpp, Some("c++17".to_string()));
    }

    #[test]
    fn parse_std_comma_separated_reversed() {
        let (c, cpp) = parse_std("c++17,c11");
        assert_eq!(c, Some("c11".to_string()));
        assert_eq!(cpp, Some("c++17".to_string()));
    }

    #[test]
    fn parse_std_single_c() {
        let (c, cpp) = parse_std("c11");
        assert_eq!(c, Some("c11".to_string()));
        assert_eq!(cpp, None);
    }

    #[test]
    fn parse_std_single_cpp() {
        let (c, cpp) = parse_std("c++17");
        assert_eq!(c, None);
        assert_eq!(cpp, Some("c++17".to_string()));
    }

    #[test]
    fn parse_std_with_spaces() {
        let (c, cpp) = parse_std("c11, c++17");
        assert_eq!(c, Some("c11".to_string()));
        assert_eq!(cpp, Some("c++17".to_string()));
    }
}
