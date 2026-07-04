use std::path::{Path, PathBuf};
use tempfile::TempDir;
use serde_json::Value;
use ccgen::types::{CcgenConfig, MacroDef, RawConfig, CompileEntry};
use ccgen::core::include_path::resolve_all;
use ccgen::{run, core::merger, output::writer};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn base_config() -> CcgenConfig {
    CcgenConfig {
        root: fixture_dir(),
        output: PathBuf::from(""),
        compiler: None,
        std_c: None,
        std_cpp: None,
        defines: vec![],
        undefs: vec![],
        include_dirs: vec![],
        include_exclude_dirs: vec![],
        source_excludes: vec![],
        no_gitignore: false,
        verbose: false,
        dry_run: false,
    }
}

fn run_and_get_entries(config: CcgenConfig) -> Vec<Value> {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("compile_commands.json");
    let mut cfg = config;
    cfg.dry_run = false;
    cfg.output = output_path.clone();
    run(cfg).unwrap();
    let content = std::fs::read_to_string(&output_path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn entry_filenames(entries: &[Value]) -> Vec<String> {
    entries.iter().map(|e| {
        let path: &str = e["file"].as_str().unwrap();
        std::path::Path::new(path)
            .file_name().unwrap()
            .to_string_lossy()
            .to_string()
    }).collect()
}

fn extract_include_dirs(entry: &Value) -> Vec<String> {
    let args = entry["arguments"].as_array().unwrap();
    let mut dirs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].as_str() == Some("-I") {
            if i + 1 < args.len() {
                dirs.push(args[i + 1].as_str().unwrap().to_string());
                i += 1;
            }
        }
        i += 1;
    }
    dirs
}

#[test]
fn basic_generation() {
    let entries = run_and_get_entries(base_config());
    assert_eq!(entries.len(), 5, "should find 5 source files");

    let names = entry_filenames(&entries);
    for name in &["main.c", "utils.cpp", "helper.cc", "core.cxx", "module.c"] {
        assert!(names.contains(&name.to_string()), "missing {name}");
    }
}

#[test]
fn macro_defines() {
    let mut config = base_config();
    config.defines = vec![
        MacroDef { name: "DEBUG".into(), value: None },
        MacroDef { name: "VERSION".into(), value: Some("42".into()) },
    ];

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5);

    for entry in &entries {
        let args: Vec<&str> = entry["arguments"]
            .as_array().unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        let d_idx = args.iter().position(|&a| a == "-D").unwrap();
        assert_eq!(args[d_idx + 1], "DEBUG");

        let d2_idx = args.iter().rposition(|&a| a == "-D").unwrap();
        assert_eq!(args[d2_idx + 1], "VERSION=42");
    }
}

#[test]
fn include_paths() {
    let mut config = base_config();
    config.include_dirs = vec![PathBuf::from("lib/include")];

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5);

    let dirs = extract_include_dirs(&entries[0]);
    assert!(dirs.iter().any(|d| d.ends_with("lib/include")),
        "should contain lib/include: {:?}", dirs);
    assert!(dirs.iter().any(|d| d.ends_with("lib/include/detail")),
        "should recursively find lib/include/detail: {:?}", dirs);
}

#[test]
fn exclude_source() {
    let mut config = base_config();
    config.source_excludes = vec!["**/sub/*".into()];

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 4, "should exclude sub/module.c");

    let names = entry_filenames(&entries);
    assert!(names.contains(&"main.c".into()));
    assert!(!names.contains(&"module.c".into()));
}

#[test]
fn exclude_include_dir() {
    let mut config = base_config();
    config.include_dirs = vec![PathBuf::from("lib/include")];
    config.include_exclude_dirs = vec![fixture_dir().join("lib/include/detail")];

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5);

    let dirs = extract_include_dirs(&entries[0]);
    assert!(dirs.iter().any(|d| d.ends_with("lib/include")),
        "should contain lib/include");
    assert!(dirs.iter().all(|d| !d.ends_with("lib/include/detail")),
        "should exclude detail subdirectory: {:?}", dirs);
}

#[test]
fn compiler_override() {
    let mut config = base_config();
    config.compiler = Some("clang".into());

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5);

    for entry in &entries {
        let first = entry["arguments"][0].as_str().unwrap();
        assert_eq!(first, "clang", "compiler should be clang, got {first}");
    }
}

#[test]
fn language_standard() {
    let mut config = base_config();
    config.std_c = Some("c17".into());

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5);

    for entry in &entries {
        let args: Vec<&str> = entry["arguments"]
            .as_array().unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let lang = args.iter().position(|&a| a == "-x").map(|i| args[i + 1]);
        let std_arg = args.iter().find(|a| a.starts_with("-std="));

        match lang {
            Some("c") => assert_eq!(std_arg, Some(&"-std=c17"),
                "C file should use -std=c17: {:?}", args),
            Some("c++") => assert_eq!(std_arg, Some(&"-std=gnu++11"),
                "C++ file should use default -std=gnu++11: {:?}", args),
            _ => {}
        }
    }
}

#[test]
fn no_gitignore() {
    let mut config = base_config();
    config.no_gitignore = true;

    let entries = run_and_get_entries(config);
    assert_eq!(entries.len(), 5, "temp.log is not a source file, so count unchanged");

    let names = entry_filenames(&entries);
    for name in &["main.c", "utils.cpp", "helper.cc", "core.cxx", "module.c"] {
        assert!(names.contains(&name.to_string()), "missing {name}");
    }
}

#[test]
fn dry_run_does_not_write_file() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("compile_commands.json");

    let mut config = base_config();
    config.dry_run = true;
    config.output = output_path.clone();

    run(config).unwrap();
    assert!(!output_path.exists(), "dry_run should not create output file");
}

#[test]
fn absolute_paths_forward_slashes() {
    let entries = run_and_get_entries(base_config());
    assert!(!entries.is_empty());

    for entry in &entries {
        let dir = entry["directory"].as_str().unwrap();
        let file = entry["file"].as_str().unwrap();

        assert!(std::path::Path::new(dir).is_absolute(),
            "directory should be absolute: {dir}");
        assert!(std::path::Path::new(file).is_absolute(),
            "file should be absolute: {file}");

            if cfg!(not(windows)) {
            assert!(!dir.contains('\\'), "use forward slashes: {dir}");
            assert!(!file.contains('\\'), "use forward slashes: {file}");
        }
    }
}

#[test]
fn atomic_write_cleans_temp() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("compile_commands.json");

    let entries = vec![CompileEntry {
        directory: PathBuf::from("/project"),
        file: PathBuf::from("/project/src/main.c"),
        arguments: vec!["gcc".into(), "-c".into(), "main.c".into()],
    }];

    writer::write_to_json(&entries, &path).unwrap();

    assert!(path.exists(), "final file should exist");
    assert!(!path.with_extension("tmp").exists(),
        "temp file should be cleaned up after atomic write");

    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: Vec<Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["file"], "/project/src/main.c");
}

#[test]
fn merge_priority() {
    let cli = RawConfig {
        compiler: Some("clang".into()),
        std_c: Some("c17".into()),
        std_cpp: Some("c++20".into()),
        defines: vec!["DEBUG=1".into()],
        ..Default::default()
    };

    let file = RawConfig {
        compiler: Some("gcc".into()),
        std_c: Some("gnu11".into()),
        std_cpp: Some("gnu++11".into()),
        includes: vec!["file_inc".into()],
        ..Default::default()
    };

    let result = merger::merge(cli, file);

    assert_eq!(result.compiler, Some("clang".into()),
        "CLI compiler should override file compiler");
    assert_eq!(result.std_c, Some("c17".into()),
        "CLI std_c should override file std_c");
    assert_eq!(result.std_cpp, Some("c++20".into()),
        "CLI std_cpp should override file std_cpp");
    assert!(result.defines.iter().any(|d| d.name == "DEBUG" && d.value == Some("1".into())),
        "CLI defines should be present");
}

#[test]
fn empty_sources_warning() {
    let mut config = base_config();
    config.root = fixture_dir().join("empty_dir");

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("compile_commands.json");
    config.output = output_path.clone();
    config.dry_run = false;

    run(config).unwrap();

    assert!(output_path.exists(), "output file should be created even with no sources");
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.trim(), "[]", "empty sources should produce empty JSON array");
}

fn default_config(root: &Path) -> CcgenConfig {
    CcgenConfig {
        root: root.to_path_buf(),
        compiler: None,
        std_c: None,
        std_cpp: None,
        defines: vec![],
        undefs: vec![],
        include_dirs: vec![],
        include_exclude_dirs: vec![],
        source_excludes: vec![],
        no_gitignore: false,
        output: PathBuf::from("out.json"),
        verbose: false,
        dry_run: false,
    }
}

#[test]
fn include_filter_basic() {
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/include_filter");
    let config = CcgenConfig {
        root: tmp.clone(),
        include_dirs: vec![PathBuf::from("has_headers")],
        include_exclude_dirs: vec![],
        ..default_config(&tmp)
    };
    let result = resolve_all(&config);
    let result_strs: Vec<String> = result.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    assert!(result_strs.iter().any(|s| s.ends_with("has_headers")), "parent must be retained: {:?}", result_strs);
    assert!(result_strs.iter().any(|s| s.ends_with("has_headers/sub")), "sub with a.h must be retained: {:?}", result_strs);
    assert!(!result_strs.iter().any(|s| s.ends_with("empty")), "empty dir must be discarded: {:?}", result_strs);
}

#[test]
fn include_filter_nested() {
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/include_filter");
    let config = CcgenConfig {
        root: tmp.clone(),
        include_dirs: vec![PathBuf::from("lib")],
        include_exclude_dirs: vec![],
        ..default_config(&tmp)
    };
    let result = resolve_all(&config);
    let result_strs: Vec<String> = result.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    assert!(result_strs.iter().any(|s| s.ends_with("lib/core/internal")), "internal must be retained: {:?}", result_strs);
    assert!(result_strs.iter().any(|s| s.ends_with("/lib/core") && !s.ends_with("internal")), "lib/core must be retained: {:?}", result_strs);
}

#[test]
fn include_filter_no_headers_discarded() {
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/include_filter");
    let config = CcgenConfig {
        root: tmp.clone(),
        include_dirs: vec![PathBuf::from("no_headers")],
        include_exclude_dirs: vec![],
        ..default_config(&tmp)
    };
    let result = resolve_all(&config);
    assert!(result.is_empty(), "no_headers should be empty: {:?}", result);
}

#[test]
fn include_filter_exclude_priority() {
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/include_filter");
    let exclude_dir = tmp.join("has_headers/sub");
    let config = CcgenConfig {
        root: tmp.clone(),
        include_dirs: vec![PathBuf::from("has_headers")],
        include_exclude_dirs: vec![exclude_dir],
        ..default_config(&tmp)
    };
    let result = resolve_all(&config);
    let result_strs: Vec<String> = result.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    assert!(!result_strs.iter().any(|s| s.ends_with("sub")), "excluded sub must not appear: {:?}", result_strs);
}

#[test]
fn include_filter_multiple_dirs() {
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/include_filter");
    let config = CcgenConfig {
        root: tmp.clone(),
        include_dirs: vec![PathBuf::from("has_headers"), PathBuf::from("lib")],
        include_exclude_dirs: vec![],
        ..default_config(&tmp)
    };
    let result = resolve_all(&config);
    let result_strs: Vec<String> = result.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    assert!(result_strs.iter().any(|s| s.ends_with("has_headers/sub")), "has_headers/sub must be in result");
    assert!(result_strs.iter().any(|s| s.ends_with("lib/core/internal")), "lib/core/internal must be in result");
}
