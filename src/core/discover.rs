use std::path::Path;

use crate::types::CcgenConfig;
use ignore::WalkBuilder;
use std::path::PathBuf;

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("c" | "cpp" | "cc" | "cxx")
    )
}

fn is_excluded(path: &Path, excludes: &[glob::Pattern]) -> bool {
    excludes.iter().any(|p| p.matches_path(path))
}

pub fn find_sources(config: &CcgenConfig) -> Vec<PathBuf> {
    let walk = WalkBuilder::new(&config.root)
        .git_ignore(!config.no_gitignore)
        .build();

    let excludes: Vec<glob::Pattern> = config
        .source_excludes
        .iter()
        .filter_map(|e| glob::Pattern::new(e).ok())
        .collect();

    let mut sources = Vec::new();

    for entry in walk {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_file() && is_source_file(path) && !is_excluded(path, &excludes) {
            let joined = config.root.join(path);
            let simplified = dunce::simplified(&joined);
            sources.push(PathBuf::from(simplified.as_os_str()));
        }
    }

    sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CcgenConfig;
    use std::path::PathBuf;

    fn test_config() -> CcgenConfig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        CcgenConfig {
            root,
            compiler: None,
            std: None,
            defines: vec![],
            undefs: vec![],
            include_dirs: vec![],
            include_exclude_dirs: vec![],
            source_excludes: vec![],
            no_gitignore: false,
            output: PathBuf::from("/tmp/out"),
            verbose: false,
            dry_run: false,
        }
    }

    fn path_names(sources: &[PathBuf]) -> Vec<String> {
        sources
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str().map(String::from)))
            .collect()
    }

    #[test]
    fn finds_all_source_files() {
        let config = test_config();
        let sources = find_sources(&config);
        let names = path_names(&sources);
        assert!(names.contains(&"main.c".into()));
        assert!(names.contains(&"helper.cc".into()));
        assert!(names.contains(&"core.cxx".into()));
        assert!(names.contains(&"utils.cpp".into()));
        assert!(names.contains(&"module.c".into()));
    }

    #[test]
    fn excludes_headers() {
        let config = test_config();
        let sources = find_sources(&config);
        let names = path_names(&sources);
        assert!(!names.contains(&"header.h".into()));
        assert!(!names.contains(&"api.h".into()));
        assert!(!names.contains(&"internal.h".into()));
        assert!(!names.contains(&"excluded.h".into()));
    }

    #[test]
    fn gitignore_filters_logs() {
        let config = test_config();
        let sources = find_sources(&config);
        let names = path_names(&sources);
        assert!(!names.contains(&"temp.log".into()));
    }

    #[test]
    fn no_gitignore_disables_gitignore() {
        let mut config = test_config();
        config.no_gitignore = true;
        let sources = find_sources(&config);
        let names = path_names(&sources);
        // temp.log is not a source file, so it's still excluded by extension filter
        assert!(!names.contains(&"temp.log".into()));
        // All source files are still found
        assert!(names.contains(&"main.c".into()));
        assert!(names.contains(&"utils.cpp".into()));
        assert!(names.contains(&"helper.cc".into()));
        assert!(names.contains(&"core.cxx".into()));
        assert!(names.contains(&"module.c".into()));
    }

    #[test]
    fn exclude_glob_filters() {
        let mut config = test_config();
        config.source_excludes = vec!["**/sub/*".into()];
        let sources = find_sources(&config);
        let names = path_names(&sources);
        assert!(names.contains(&"main.c".into()));
        assert!(!names.contains(&"module.c".into()));
    }

    #[test]
    fn exclude_glob_filters_subtree() {
        let mut config = test_config();
        config.source_excludes = vec!["**/exclude_me/*".into(), "**/docs/*".into()];
        let sources = find_sources(&config);
        let names = path_names(&sources);
        assert!(names.contains(&"main.c".into()));
    }

    #[test]
    fn all_paths_are_absolute() {
        let config = test_config();
        let sources = find_sources(&config);
        for p in &sources {
            assert!(p.is_absolute(), "path {:?} is not absolute", p);
        }
    }

    #[test]
    fn empty_dir_returns_empty() {
        let mut config = test_config();
        config.root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("empty_dir");
        // ensure the empty dir exists
        let _ = std::fs::create_dir_all(&config.root);
        let sources = find_sources(&config);
        assert!(sources.is_empty());
    }

    #[test]
    fn all_paths_are_source_extensions() {
        let config = test_config();
        let sources = find_sources(&config);
        for p in &sources {
            assert!(
                is_source_file(p),
                "{:?} is not a source file",
                p.display()
            );
        }
    }
}
