use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crate::types::CcgenConfig;

pub fn resolve_all(config: &CcgenConfig) -> Vec<PathBuf> {
    let mut result = Vec::new();

    for dir in &config.include_dirs {
        let abs_dir = if dir.is_absolute() {
            dir.clone()
        } else {
            config.root.join(dir)
        };
        collect_dirs(&abs_dir, &config.include_exclude_dirs, &mut result);
    }

    result.sort();
    result.dedup();
    result
}

fn collect_dirs(root: &Path, exclude_dirs: &[PathBuf], result: &mut Vec<PathBuf>) {
    let mut queue = VecDeque::new();
    queue.push_back(root.to_path_buf());

    while let Some(dir) = queue.pop_front() {
        if is_excluded_dir(&dir, exclude_dirs) {
            continue;
        }
        result.push(normalize_path(&dir));

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    queue.push_back(entry.path());
                }
            }
        }
    }
}

fn is_excluded_dir(path: &Path, exclude_dirs: &[PathBuf]) -> bool {
    exclude_dirs.iter().any(|ex| path.starts_with(ex) || path == ex)
}

fn normalize_path(path: &Path) -> PathBuf {
    let abs = dunce::simplified(path);
    let s = abs.to_string_lossy().replace('\\', "/");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_tree(root: &Path) {
        fs::create_dir_all(root.join("dir_a")).unwrap();
        fs::create_dir_all(root.join("dir_a/sub_1")).unwrap();
        fs::create_dir_all(root.join("dir_a/sub_1/deep")).unwrap();
        fs::create_dir_all(root.join("dir_a/sub_2")).unwrap();
        fs::create_dir_all(root.join("dir_b")).unwrap();
        fs::create_dir_all(root.join("dir_c")).unwrap();
        // put a file in dir_a to ensure we only collect dirs
        File::create(root.join("dir_a/foo.txt")).unwrap();
    }

    #[test]
    fn bfs_discovers_all_subdirectories() {
        let tmp = TempDir::new().unwrap();
        create_tree(tmp.path());

        let mut result = Vec::new();
        collect_dirs(&tmp.path().join("dir_a"), &[], &mut result);

        let names: Vec<String> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"dir_a".into()));
        assert!(names.contains(&"sub_1".into()));
        assert!(names.contains(&"deep".into()));
        assert!(names.contains(&"sub_2".into()));
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn exclude_dir_omits_directory_and_children() {
        let tmp = TempDir::new().unwrap();
        create_tree(tmp.path());

        let exclude = vec![tmp.path().join("dir_a/sub_1")];
        let mut result = Vec::new();
        collect_dirs(&tmp.path().join("dir_a"), &exclude, &mut result);

        let names: Vec<String> = result
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"dir_a".into()));
        assert!(names.contains(&"sub_2".into()));
        assert!(!names.contains(&"sub_1".into()));
        assert!(!names.contains(&"deep".into()));
    }

    #[test]
    fn normalize_path_uses_forward_slashes() {
        let p = normalize_path(&PathBuf::from(r"C:\Users\test\path"));
        let s = p.to_string_lossy();
        assert!(!s.contains('\\'), "path must use forward slashes: {s}");
        // On Windows the path starts with C:/, on Unix it stays as-is (relative path)
        if cfg!(windows) {
            assert!(s.starts_with("C:/"), "path should start with drive letter: {s}");
        }
    }

    #[test]
    fn resolve_all_empty_list_returns_empty() {
        let config = CcgenConfig {
            root: PathBuf::from("."),
            compiler: None,
            std: None,
            defines: vec![],
            undefs: vec![],
            include_dirs: vec![],
            include_exclude_dirs: vec![],
            source_excludes: vec![],
            no_gitignore: false,
            output: PathBuf::from("out"),
            verbose: false,
            dry_run: false,
        };
        let result = resolve_all(&config);
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_all_relative_dirs_resolved_against_root() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("inc");
        fs::create_dir_all(&sub).unwrap();
        // need a parent dir so root.join("inc") resolves to sub
        let root_dir = tmp.path().join("root");
        fs::create_dir_all(&root_dir).unwrap();
        // but we make include relative and root point to tmp
        let rel = PathBuf::from("inc");
        let inc_abs = tmp.path().join(&rel);
        fs::create_dir_all(&inc_abs).unwrap();

        let config = CcgenConfig {
            root: tmp.path().to_path_buf(),
            compiler: None,
            std: None,
            defines: vec![],
            undefs: vec![],
            include_dirs: vec![rel],
            include_exclude_dirs: vec![],
            source_excludes: vec![],
            no_gitignore: false,
            output: PathBuf::from("out"),
            verbose: false,
            dry_run: false,
        };
        let result = resolve_all(&config);
        assert_eq!(result.len(), 1);
        let s = result[0].to_string_lossy();
        assert!(!s.contains('\\'), "must have forward slashes: {s}");
        assert!(s.contains("inc"), "{s} must contain 'inc'");
    }
}
