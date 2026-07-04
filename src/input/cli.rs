use clap::Parser;
use std::path::PathBuf;

use crate::types::RawConfig;

/// C/C++ compile_commands.json generator
#[derive(Parser, Debug)]
#[command(name = "ccgen", version, about)]
pub struct CliArgs {
    /// Project root directory (default: current directory)
    #[arg(default_value = ".")]
    pub root: PathBuf,

    /// Macro definitions: -D NAME or -D NAME=VALUE
    #[arg(short = 'D', long = "define", value_name = "KEY=VALUE", action = clap::ArgAction::Append)]
    pub defines: Vec<String>,

    /// Macro undefines: -U NAME
    #[arg(short = 'U', long = "undef", value_name = "NAME", action = clap::ArgAction::Append)]
    pub undefs: Vec<String>,

    /// Include search paths
    #[arg(short = 'I', long = "include", value_name = "PATH", action = clap::ArgAction::Append)]
    pub includes: Vec<String>,

    /// Source file exclude glob patterns
    #[arg(long = "exclude", value_name = "PATTERN", action = clap::ArgAction::Append)]
    pub excludes: Vec<String>,

    /// Include subdirectory excludes
    #[arg(long = "exclude-dir", value_name = "DIR", action = clap::ArgAction::Append)]
    pub exclude_dirs: Vec<String>,

    /// Override the detected compiler
    #[arg(long = "compiler", value_name = "NAME")]
    pub compiler: Option<String>,

    /// Language standard (e.g. c11, c17, c++20)
    #[arg(long = "std", value_name = "STD")]
    pub std: Option<String>,

    /// Disable .gitignore filtering
    #[arg(long = "no-gitignore")]
    pub no_gitignore: bool,

    /// Output file path
    #[arg(short = 'o', long = "output", value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Path to config file
    #[arg(long = "config", value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Dry run: print result to stdout instead of writing to file
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Generate default .ccgen.toml config file template
    #[arg(long = "init-config")]
    pub init_config: bool,
}

impl CliArgs {
    pub fn to_raw_config(self) -> RawConfig {
        RawConfig {
            root: self.root,
            compiler: self.compiler,
            std: self.std,
            defines: self.defines,
            undefs: self.undefs,
            includes: self.includes,
            excludes: self.excludes,
            exclude_dirs: self.exclude_dirs,
            no_gitignore: self.no_gitignore,
            output: self.output,
            verbose: self.verbose,
            dry_run: self.dry_run,
        }
    }
}

pub fn parse_args() -> RawConfig {
    CliArgs::parse().to_raw_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let args = CliArgs::try_parse_from(["ccgen"]).unwrap();
        assert_eq!(args.root, PathBuf::from("."));
        assert!(args.defines.is_empty());
        assert!(args.undefs.is_empty());
        assert!(args.includes.is_empty());
        assert!(args.excludes.is_empty());
        assert!(args.exclude_dirs.is_empty());
        assert!(args.compiler.is_none());
        assert!(args.std.is_none());
        assert!(!args.no_gitignore);
        assert!(args.output.is_none());
        assert!(args.config.is_none());
        assert!(!args.dry_run);
        assert!(!args.verbose);
    }

    #[test]
    fn test_root_positional_arg() {
        let args = CliArgs::try_parse_from(["ccgen", "/some/path"]).unwrap();
        assert_eq!(args.root, PathBuf::from("/some/path"));
    }

    #[test]
    fn test_define_name_only() {
        let args = CliArgs::try_parse_from(["ccgen", "-D", "FOO"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.defines, vec!["FOO"]);
    }

    #[test]
    fn test_define_name_value() {
        let args = CliArgs::try_parse_from(["ccgen", "-D", "FOO=bar"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.defines, vec!["FOO=bar"]);
    }

    #[test]
    fn test_define_spaced_value() {
        let args = CliArgs::try_parse_from(["ccgen", "-D", "NAME=spaced value"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.defines, vec!["NAME=spaced value"]);
    }

    #[test]
    fn test_define_multiple() {
        let args = CliArgs::try_parse_from(["ccgen", "-D", "FOO", "-D", "BAR=baz", "-D", "DEBUG=1"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.defines, vec!["FOO", "BAR=baz", "DEBUG=1"]);
    }

    #[test]
    fn test_undefines() {
        let args = CliArgs::try_parse_from(["ccgen", "-U", "FOO", "-U", "BAR"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.undefs, vec!["FOO", "BAR"]);
    }

    #[test]
    fn test_includes() {
        let args = CliArgs::try_parse_from(["ccgen", "-I", "/path/one", "-I", "/path/two"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.includes, vec!["/path/one", "/path/two"]);
    }

    #[test]
    fn test_exclude_and_exclude_dir() {
        let args = CliArgs::try_parse_from([
            "ccgen", "--exclude", "*.test.*", "--exclude-dir", "test",
        ]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.excludes, vec!["*.test.*"]);
        assert_eq!(cfg.exclude_dirs, vec!["test"]);
    }

    #[test]
    fn test_compiler_and_std() {
        let args = CliArgs::try_parse_from(["ccgen", "--compiler", "clang", "--std", "c17"]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.compiler, Some("clang".to_string()));
        assert_eq!(cfg.std, Some("c17".to_string()));
    }

    #[test]
    fn test_boolean_flags() {
        let args = CliArgs::try_parse_from(["ccgen", "--no-gitignore", "--dry-run", "--verbose"]).unwrap();
        assert!(args.no_gitignore);
        assert!(args.dry_run);
        assert!(args.verbose);
    }

    #[test]
    fn test_output_and_config() {
        let args = CliArgs::try_parse_from([
            "ccgen", "-o", "output.txt", "--config", "config.toml",
        ]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.output, Some(PathBuf::from("output.txt")));
    }

    #[test]
    fn test_to_raw_config_complete() {
        let args = CliArgs::try_parse_from([
            "ccgen",
            "/project/root",
            "-D", "DEBUG",
            "-D", "VERSION=2",
            "-U", "OLD",
            "-I", "/usr/include",
            "-I", "/usr/local/include",
            "--exclude", "*.test.*",
            "--exclude-dir", "test",
            "--compiler", "gcc",
            "--std", "c11",
            "--no-gitignore",
            "--dry-run",
            "--verbose",
            "-o", "/tmp/output.json",
            "--config", "./ccgen.toml",
        ]).unwrap();
        let cfg = args.to_raw_config();
        assert_eq!(cfg.root, PathBuf::from("/project/root"));
        assert_eq!(cfg.defines, vec!["DEBUG", "VERSION=2"]);
        assert_eq!(cfg.undefs, vec!["OLD"]);
        assert_eq!(cfg.includes, vec!["/usr/include", "/usr/local/include"]);
        assert_eq!(cfg.excludes, vec!["*.test.*"]);
        assert_eq!(cfg.exclude_dirs, vec!["test"]);
        assert_eq!(cfg.compiler, Some("gcc".to_string()));
        assert_eq!(cfg.std, Some("c11".to_string()));
        assert!(cfg.no_gitignore);
        assert_eq!(cfg.output, Some(PathBuf::from("/tmp/output.json")));
    }

    #[test]
    fn test_short_flag_combos() {
        let args = CliArgs::try_parse_from([
            "ccgen", "-D", "A", "-U", "B", "-I", "dir",
        ]).unwrap();
        assert_eq!(args.defines, vec!["A"]);
        assert_eq!(args.undefs, vec!["B"]);
        assert_eq!(args.includes, vec!["dir"]);
    }
}
