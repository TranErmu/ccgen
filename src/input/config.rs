use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use serde::Deserialize;

use crate::error::CcgenError;
use crate::types::{parse_std, RawConfig};

#[derive(Deserialize)]
struct TomlConfig {
    compiler: Option<String>,
    std: Option<String>,
    defines: Option<Vec<String>>,
    undefs: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    exclude_dir: Option<Vec<String>>,
    no_gitignore: Option<bool>,
}

pub fn find(root: &Path) -> Option<PathBuf> {
    let candidate = root.join(".ccgen.toml");
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

pub fn parse(path: &Path) -> anyhow::Result<RawConfig> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let toml_config: TomlConfig =
        toml::from_str(&content).map_err(|e| CcgenError::Config(format!("TOML parse error: {}", e)))?;
    Ok(toml_config.into_raw_config())
}

pub fn default_toml_content() -> &'static str {
    r#"# ccgen configuration file
# All options are optional. CLI arguments override config file values.

# Compiler to use (auto-detected if omitted)
# compiler = "gcc"

# Language standard (e.g. c11, c17, c++20, or comma-separated c11,c++17)
# std = "c17,c++20"

# Preprocessor defines: -D NAME or -D NAME=VALUE
# defines = ["DEBUG", "VERSION=1"]

# Preprocessor undefines: -U NAME
# undefs = ["OLD_MACRO"]

# Include search paths (relative to root or absolute)
# include = ["src", "include"]

# Source file exclude glob patterns
# exclude = ["*.test.*", "*.spec.*"]

# Include subdirectory excludes (relative to root)
# exclude_dir = [".git", "target", "build"]

# Disable .gitignore filtering
# no_gitignore = false
"#
}

pub fn write_default_config(root: &Path) -> anyhow::Result<()> {
    let config_path = root.join(".ccgen.toml");
    if config_path.exists() {
        bail!("Error: .ccgen.toml already exists in {}", root.display());
    }
    std::fs::write(&config_path, default_toml_content())
        .with_context(|| format!("Failed to write {}", config_path.display()))?;
    eprintln!("Created .ccgen.toml in {}", root.display());
    Ok(())
}

impl TomlConfig {
    fn into_raw_config(self) -> RawConfig {
        let (std_c, std_cpp) = match self.std {
            Some(ref v) => parse_std(v),
            None => (None, None),
        };
        RawConfig {
            compiler: self.compiler,
            std_c,
            std_cpp,
            defines: self.defines.unwrap_or_default(),
            undefs: self.undefs.unwrap_or_default(),
            includes: self.include.unwrap_or_default(),
            excludes: self.exclude.unwrap_or_default(),
            exclude_dirs: self.exclude_dir.unwrap_or_default(),
            no_gitignore: self.no_gitignore.unwrap_or(false),
            root: PathBuf::from("."),
            output: None,
            verbose: false,
            dry_run: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn write_temp_file(content: &str) -> PathBuf {
        let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("__ccgen_test_config_{}.toml", count));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_find_not_found() {
        let tmp = std::env::temp_dir().join("__ccgen_test_nonexistent_dir__");
        let _ = std::fs::create_dir_all(&tmp);
        assert!(find(&tmp).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_find_found() {
        let dir = std::env::temp_dir().join("__ccgen_test_find_found__");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join(".ccgen.toml");
        std::fs::write(&config_path, "").unwrap();
        assert!(find(&dir).is_some());
        std::fs::remove_file(&config_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_parse_complete_config() {
        let toml = r#"
compiler = "gcc"
std = "c11"
defines = ["FOO", "BAR=1"]
undefs = ["OLD"]
include = ["src", "include"]
exclude = ["test", "build"]
exclude_dir = [".git", "target"]
no_gitignore = true
"#;
        let config = parse_from_str(toml).unwrap();
        assert_eq!(config.compiler, Some("gcc".to_string()));
        assert_eq!(config.std_c, Some("c11".to_string()));
        assert!(config.std_cpp.is_none());
        assert_eq!(config.defines, vec!["FOO", "BAR=1"]);
        assert_eq!(config.undefs, vec!["OLD"]);
        assert_eq!(config.includes, vec!["src", "include"]);
        assert_eq!(config.excludes, vec!["test", "build"]);
        assert_eq!(config.exclude_dirs, vec![".git", "target"]);
        assert!(config.no_gitignore);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
compiler = "clang"
include = ["src"]
"#;
        let config = parse_from_str(toml).unwrap();
        assert_eq!(config.compiler, Some("clang".to_string()));
        assert_eq!(config.includes, vec!["src"]);
        assert!(config.defines.is_empty());
        assert!(config.undefs.is_empty());
        assert!(config.excludes.is_empty());
        assert!(config.exclude_dirs.is_empty());
        assert!(!config.no_gitignore);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config = parse_from_str(toml).unwrap();
        assert!(config.compiler.is_none());
        assert!(config.std_c.is_none());
        assert!(config.std_cpp.is_none());
        assert!(config.defines.is_empty());
        assert!(config.undefs.is_empty());
        assert!(config.includes.is_empty());
        assert!(config.excludes.is_empty());
        assert!(config.exclude_dirs.is_empty());
        assert!(!config.no_gitignore);
    }

    #[test]
    fn test_parse_malformed_toml() {
        let toml = "this is not valid toml =====";
        let result = parse_from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_not_found() {
        let path = Path::new("C:\\__ccgen_test_nonexistent_file__\\.ccgen.toml");
        let result = parse(path);
        assert!(result.is_err());
    }

    fn parse_from_str(toml: &str) -> anyhow::Result<RawConfig> {
        let path = write_temp_file(toml);
        let result = parse(&path);
        std::fs::remove_file(&path).ok();
        result
    }

    #[test]
    fn test_default_toml_content_not_empty() {
        let content = default_toml_content();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_default_toml_content_contains_all_fields() {
        let content = default_toml_content();
        assert!(content.contains("compiler"));
        assert!(content.contains("std"));
        assert!(content.contains("defines"));
        assert!(content.contains("undefs"));
        assert!(content.contains("include"));
        assert!(content.contains("exclude"));
        assert!(content.contains("exclude_dir"));
        assert!(content.contains("no_gitignore"));
    }

    #[test]
    fn test_write_default_config_success() {
        let dir = std::env::temp_dir().join("__ccgen_test_init_config_ok__");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join(".ccgen.toml");
        let _ = std::fs::remove_file(&config_path);

        let result = write_default_config(&dir);
        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("compiler"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_write_default_config_already_exists() {
        let dir = std::env::temp_dir().join("__ccgen_test_init_config_exists__");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join(".ccgen.toml");
        std::fs::write(&config_path, "existing").unwrap();

        let result = write_default_config(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
