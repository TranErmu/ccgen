use std::collections::{HashMap, HashSet, VecDeque};
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

    result = filter_by_headers(&result);

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

fn is_header_file(entry: &std::fs::DirEntry) -> bool {
    if !entry.path().is_file() {
        return false;
    }
    match entry.path().extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "h" | "hh" | "hpp" | "hxx" | "h++" | "ipp" | "tcc" | "inl"
        ),
        None => false,
    }
}

fn has_header_files_in_dir(path: &Path) -> std::io::Result<bool> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(false),
    };
    for entry in entries.flatten() {
        if is_header_file(&entry) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn filter_by_headers(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut cache: HashMap<PathBuf, bool> = HashMap::new();

    let mut sorted_dirs: Vec<PathBuf> = dirs.to_vec();
    sorted_dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

    let mut results: HashSet<PathBuf> = HashSet::new();

    for dir in &sorted_dirs {
        let has_headers = {
            if let Some(&cached) = cache.get(dir) {
                cached
            } else {
                let found = has_header_files_in_dir(dir).unwrap_or(false);
                cache.insert(dir.clone(), found);
                found
            }
        };

        if has_headers {
            results.insert(dir.clone());
            for other in dirs {
                if dir.starts_with(other) && !results.contains(other) {
                    results.insert(other.clone());
                }
            }
        }
    }

    let mut result: Vec<PathBuf> = results.into_iter().collect();
    result.sort();
    result
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
        // add a header file so the directory passes header filtering
        File::create(sub.join("types.h")).unwrap();
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

    #[test]
    fn test_is_header_file_various_extensions() {
        let tmp = TempDir::new().unwrap();
        for ext in &["h", "hh", "hpp", "hxx", "h++", "ipp", "tcc", "inl"] {
            let path = tmp.path().join(format!("file.{ext}"));
            File::create(&path).unwrap();
            let entry = std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .find(|e| e.path() == path)
                .unwrap();
            assert!(
                is_header_file(&entry),
                "expected {ext} to be a header"
            );
        }
    }

    #[test]
    fn test_is_header_file_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        for name in &["Foo.H", "Bar.HPP"] {
            let path = tmp.path().join(name);
            File::create(&path).unwrap();
            let entry = std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .find(|e| e.path() == path)
                .unwrap();
            assert!(is_header_file(&entry), "expected {name} to be a header");
        }
    }

    #[test]
    fn test_is_header_file_non_headers() {
        let tmp = TempDir::new().unwrap();
        for ext in &["c", "cpp", "txt", "md"] {
            let path = tmp.path().join(format!("file.{ext}"));
            File::create(&path).unwrap();
            let entry = std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|e| e.ok())
                .find(|e| e.path() == path)
                .unwrap();
            assert!(
                !is_header_file(&entry),
                "expected {ext} to NOT be a header"
            );
        }
    }

    #[test]
    fn test_is_header_file_directory() {
        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path().join("foo.h");
        fs::create_dir(&dir_path).unwrap();
        let entry = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.path() == dir_path)
            .unwrap();
        assert!(!is_header_file(&entry), "directory should not be a header");
    }

    #[test]
    fn test_has_header_files_empty_dir() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_header_files_in_dir(tmp.path()).unwrap());
    }

    #[test]
    fn test_has_header_files_with_headers() {
        let tmp = TempDir::new().unwrap();
        File::create(tmp.path().join("foo.h")).unwrap();
        assert!(has_header_files_in_dir(tmp.path()).unwrap());
    }

    #[test]
    fn test_filter_by_headers_basic() {
        let tmp = TempDir::new().unwrap();
        let dir_a = tmp.path().join("a");
        let dir_b = tmp.path().join("b");
        let dir_c = tmp.path().join("c");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();
        fs::create_dir(&dir_c).unwrap();
        File::create(dir_a.join("x.h")).unwrap();
        File::create(dir_b.join("x.c")).unwrap();

        let result = filter_by_headers(&[dir_a.clone(), dir_b, dir_c]);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&dir_a));
    }

    #[test]
    fn test_filter_by_headers_nested() {
        let tmp = TempDir::new().unwrap();
        let parent = tmp.path().join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();
        File::create(child.join("x.h")).unwrap();

        let result = filter_by_headers(&[parent.clone(), child.clone()]);
        assert!(result.contains(&child));
        assert!(result.contains(&parent));
    }

    #[test]
    fn test_filter_by_headers_empty_branch() {
        let tmp = TempDir::new().unwrap();
        let branch = tmp.path().join("empty").join("deep");
        fs::create_dir_all(&branch).unwrap();

        let result = filter_by_headers(&[branch]);
        assert!(result.is_empty());
    }
}
