use std::path::PathBuf;

use crate::types::{CcgenConfig, MacroDef, RawConfig};

pub fn merge(cli: RawConfig, file: RawConfig) -> CcgenConfig {
    let root = if cli.root.as_os_str() == "." {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        cli.root.clone()
    };
    let root = dunce::canonicalize(&root).unwrap_or(root);

    let verbose = cli.verbose || file.verbose;
    let dry_run = cli.dry_run || file.dry_run;

    let cli_compiler = cli.compiler.as_ref().cloned();
    let file_compiler = file.compiler.as_ref().cloned();
    let cli_std = cli.std.as_ref().cloned();
    let file_std = file.std.as_ref().cloned();

    let compiler = cli.compiler.or(file.compiler);
    let std = cli.std.or(file.std);
    let no_gitignore = cli.no_gitignore || file.no_gitignore;

    let output = cli
        .output
        .clone()
        .or(file.output)
        .unwrap_or_else(|| root.join("compile_commands.json"));

    let defines = merge_defines(&cli.defines, &file.defines);
    let undefs = if cli.undefs.is_empty() {
        file.undefs
    } else {
        cli.undefs
    };
    let include_dirs = if cli.includes.is_empty() {
        file.includes.iter().map(PathBuf::from).collect()
    } else {
        cli.includes.iter().map(PathBuf::from).collect()
    };
    let include_exclude_dirs = if cli.exclude_dirs.is_empty() {
        file.exclude_dirs.iter().map(PathBuf::from).collect()
    } else {
        cli.exclude_dirs.iter().map(PathBuf::from).collect()
    };
    let source_excludes = if cli.excludes.is_empty() {
        file.excludes
    } else {
        cli.excludes
    };

    let config = CcgenConfig {
        root,
        compiler,
        std,
        defines,
        undefs,
        include_dirs,
        include_exclude_dirs,
        source_excludes,
        no_gitignore,
        output,
        verbose,
        dry_run,
    };

    if config.verbose {
        eprintln!(
            "[config] compiler: {:?} (from {})",
            config.compiler,
            if cli_compiler.is_some() { "CLI" } else if file_compiler.is_some() { "config file" } else { "default" }
        );
        eprintln!(
            "[config] std: {:?} (from {})",
            config.std,
            if cli_std.is_some() { "CLI" } else if file_std.is_some() { "config file" } else { "default" }
        );
        eprintln!(
            "[config] root: {:?} (from {})",
            config.root,
            if cli.root.as_os_str() != "." { "CLI" } else { "default" }
        );
    }

    config
}

fn merge_defines(cli_defines: &[String], file_defines: &[String]) -> Vec<MacroDef> {
    let mut map: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();

    for d in file_defines {
        let (name, value) = parse_define(d);
        map.insert(name, value);
    }

    for d in cli_defines {
        let (name, value) = parse_define(d);
        map.insert(name, value);
    }

    map.into_iter()
        .map(|(name, value)| MacroDef { name, value })
        .collect()
}

fn parse_define(s: &str) -> (String, Option<String>) {
    if let Some(eq) = s.find('=') {
        let name = s[..eq].to_string();
        let value = Some(s[eq + 1..].to_string());
        (name, value)
    } else {
        (s.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_raw() -> RawConfig {
        RawConfig {
            compiler: None,
            std: None,
            defines: vec![],
            undefs: vec![],
            includes: vec![],
            excludes: vec![],
            exclude_dirs: vec![],
            no_gitignore: false,
            root: PathBuf::from("."),
            output: None,
            verbose: false,
            dry_run: false,
        }
    }

    #[test]
    fn cli_overrides_file() {
        let mut cli = empty_raw();
        cli.compiler = Some("clang".to_string());

        let mut file = empty_raw();
        file.compiler = Some("gcc".to_string());

        let result = merge(cli, file);
        assert_eq!(result.compiler, Some("clang".to_string()));
    }

    #[test]
    fn file_used_when_cli_not_set() {
        let cli = empty_raw();
        let mut file = empty_raw();
        file.compiler = Some("gcc".to_string());

        let result = merge(cli, file);
        assert_eq!(result.compiler, Some("gcc".to_string()));
    }

    #[test]
    fn cli_defines_override_file_defines() {
        let mut cli = empty_raw();
        cli.defines = vec!["DEBUG=1".to_string(), "EXTRA".to_string()];

        let mut file = empty_raw();
        file.defines = vec!["DEBUG=0".to_string(), "FILE_ONLY".to_string()];

        let result = merge(cli, file);
        let debug = result.defines.iter().find(|d| d.name == "DEBUG").unwrap();
        assert_eq!(debug.value, Some("1".to_string()));

        assert!(result.defines.iter().any(|d| d.name == "EXTRA"));
        assert!(result.defines.iter().any(|d| d.name == "FILE_ONLY"));
    }

    #[test]
    fn cli_undefs_override_file_undefs() {
        let mut cli = empty_raw();
        cli.undefs = vec!["SOME_MACRO".to_string()];

        let mut file = empty_raw();
        file.undefs = vec!["OTHER_MACRO".to_string()];

        let result = merge(cli, file);
        assert_eq!(result.undefs, vec!["SOME_MACRO".to_string()]);
    }

    #[test]
    fn default_output_path() {
        let cli = empty_raw();
        let file = empty_raw();

        let result = merge(cli, file);
        assert!(result.output.to_string_lossy().ends_with("compile_commands.json"));
    }

    #[test]
    fn cli_includes_override_file() {
        let mut cli = empty_raw();
        cli.includes = vec!["cli_inc".to_string()];

        let mut file = empty_raw();
        file.includes = vec!["file_inc".to_string()];

        let result = merge(cli, file);
        assert_eq!(result.include_dirs.len(), 1);
        assert!(result.include_dirs[0].to_string_lossy().ends_with("cli_inc"));
    }

    #[test]
    fn parse_define_with_value() {
        let (name, value) = parse_define("FOO=bar");
        assert_eq!(name, "FOO");
        assert_eq!(value, Some("bar".to_string()));
    }

    #[test]
    fn parse_define_without_value() {
        let (name, value) = parse_define("FOO");
        assert_eq!(name, "FOO");
        assert_eq!(value, None);
    }

    #[test]
    fn verbose_flag_from_cli() {
        let mut cli = empty_raw();
        cli.verbose = true;
        let result = merge(cli, empty_raw());
        assert!(result.verbose);
    }
}
