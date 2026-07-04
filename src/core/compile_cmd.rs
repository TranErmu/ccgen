use std::path::{Path, PathBuf};

use crate::types::{CcgenConfig, CompileEntry};

pub fn infer_compiler(file_path: &Path) -> &str {
    match file_path.extension().and_then(|e| e.to_str()) {
        Some("c") => "gcc",
        Some("cpp") | Some("cc") | Some("cxx") => "g++",
        _ => "gcc",
    }
}

pub fn build_entry(
    config: &CcgenConfig,
    source: &Path,
    include_dirs: &[PathBuf],
) -> CompileEntry {
    let compiler = config
        .compiler
        .as_deref()
        .unwrap_or_else(|| infer_compiler(source));

    let lang = match source.extension().and_then(|e| e.to_str()) {
        Some("c") => "c",
        _ => "c++",
    };

    let mut arguments: Vec<String> = vec![
        compiler.to_string(),
        "-x".to_string(),
        lang.to_string(),
        "-c".to_string(),
        source.to_string_lossy().to_string(),
    ];

    for dir in include_dirs {
        arguments.push("-I".to_string());
        arguments.push(dir.to_string_lossy().to_string());
    }

    for def in &config.defines {
        let macro_str = match &def.value {
            Some(v) => format!("{}={}", def.name, v),
            None => def.name.clone(),
        };
        arguments.push("-D".to_string());
        arguments.push(macro_str);
    }

    for name in &config.undefs {
        arguments.push("-U".to_string());
        arguments.push(name.clone());
    }

    if let Some(std) = &config.std {
        arguments.push(format!("-std={}", std));
    }

    if config.verbose {
        eprintln!("[ccgen] compile entry: {}", source.display());
        eprintln!("[ccgen] compiler: {}", compiler);
        eprintln!("[ccgen] arguments: {:?}", arguments);
    }

    CompileEntry {
        directory: config.root.clone(),
        file: source.to_path_buf(),
        arguments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MacroDef;

    fn test_config() -> CcgenConfig {
        CcgenConfig {
            root: PathBuf::from("/project"),
            compiler: None,
            std: None,
            defines: vec![],
            undefs: vec![],
            include_dirs: vec![],
            include_exclude_dirs: vec![],
            source_excludes: vec![],
            no_gitignore: false,
            output: PathBuf::from("/out"),
            verbose: false,
            dry_run: false,
        }
    }

    macro_rules! assert_args {
        ($entry:expr, $expected:expr) => {
            let got: Vec<&str> = $entry.arguments.iter().map(|s| s.as_str()).collect();
            assert_eq!(got, $expected, "arguments mismatch");
        };
    }

    #[test]
    fn infer_compiler_c() {
        assert_eq!(infer_compiler(Path::new("foo.c")), "gcc");
    }

    #[test]
    fn infer_compiler_cpp() {
        assert_eq!(infer_compiler(Path::new("foo.cpp")), "g++");
    }

    #[test]
    fn infer_compiler_cc() {
        assert_eq!(infer_compiler(Path::new("foo.cc")), "g++");
    }

    #[test]
    fn infer_compiler_cxx() {
        assert_eq!(infer_compiler(Path::new("foo.cxx")), "g++");
    }

    #[test]
    fn infer_compiler_header_defaults_to_gcc() {
        assert_eq!(infer_compiler(Path::new("foo.h")), "gcc");
    }

    #[test]
    fn infer_compiler_no_extension_defaults_to_gcc() {
        assert_eq!(infer_compiler(Path::new("Makefile")), "gcc");
    }

    #[test]
    fn build_entry_full_arguments() {
        let mut config = test_config();
        config.defines = vec![
            MacroDef {
                name: "FOO".into(),
                value: None,
            },
            MacroDef {
                name: "BAR".into(),
                value: Some("1".into()),
            },
        ];
        config.undefs = vec!["OLD".into()];
        config.std = Some("gnu11".into());

        let source = Path::new("/project/src/main.c");
        let include_dirs = vec![PathBuf::from("/project/include")];
        let entry = build_entry(&config, source, &include_dirs);

        assert_eq!(entry.directory, PathBuf::from("/project"));
        assert_eq!(entry.file, PathBuf::from("/project/src/main.c"));
        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "/project/src/main.c",
            "-I",
            "/project/include",
            "-D",
            "FOO",
            "-D",
            "BAR=1",
            "-U",
            "OLD",
            "-std=gnu11",
        ]);
    }

    #[test]
    fn build_entry_compiler_override() {
        let mut config = test_config();
        config.compiler = Some("clang".into());

        let source = Path::new("foo.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &["clang", "-x", "c", "-c", "foo.c",]);
    }

    #[test]
    fn build_entry_cpp_language() {
        let config = test_config();
        let source = Path::new("/project/src/bar.cpp");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &[
            "g++",
            "-x",
            "c++",
            "-c",
            "/project/src/bar.cpp",
        ]);
    }

    #[test]
    fn build_entry_no_defines_undefs_std() {
        let config = test_config();
        let source = Path::new("main.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &["gcc", "-x", "c", "-c", "main.c",]);
    }

    #[test]
    fn build_entry_with_include_dirs() {
        let config = test_config();
        let source = Path::new("main.c");
        let include_dirs = vec![
            PathBuf::from("/inc1"),
            PathBuf::from("/inc2"),
        ];
        let entry = build_entry(&config, source, &include_dirs);

        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "main.c",
            "-I",
            "/inc1",
            "-I",
            "/inc2",
        ]);
    }

    #[test]
    fn build_entry_verbose_prints_logs() {
        let mut config = test_config();
        config.verbose = true;

        let source = Path::new("main.c");
        build_entry(&config, source, &[]);

        // Just ensure no panic; output is on stderr
    }

    #[test]
    fn build_entry_d_define_with_value() {
        let mut config = test_config();
        config.defines = vec![MacroDef {
            name: "VERSION".into(),
            value: Some("42".into()),
        }];

        let source = Path::new("main.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "main.c",
            "-D",
            "VERSION=42",
        ]);
    }

    #[test]
    fn build_entry_d_define_without_value() {
        let mut config = test_config();
        config.defines = vec![MacroDef {
            name: "DEBUG".into(),
            value: None,
        }];

        let source = Path::new("main.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "main.c",
            "-D",
            "DEBUG",
        ]);
    }

    #[test]
    fn build_entry_u_undef() {
        let mut config = test_config();
        config.undefs = vec!["NDEBUG".into()];

        let source = Path::new("main.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "main.c",
            "-U",
            "NDEBUG",
        ]);
    }

    #[test]
    fn build_entry_std() {
        let mut config = test_config();
        config.std = Some("c17".into());

        let source = Path::new("main.c");
        let entry = build_entry(&config, source, &[]);

        assert_args!(entry, &[
            "gcc",
            "-x",
            "c",
            "-c",
            "main.c",
            "-std=c17",
        ]);
    }
}
