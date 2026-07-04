use std::fs;
use std::path::Path;

use crate::types::CompileEntry;

pub fn write_to_json(entries: &[CompileEntry], path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let tmp_path = path.with_extension("tmp");
    let file = fs::File::create(&tmp_path)?;
    serde_json::to_writer_pretty(&file, entries)?;
    file.sync_all()?;
    fs::rename(&tmp_path, path)?;

    Ok(())
}

pub fn print_json(entries: &[CompileEntry]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(entries)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_entries() -> Vec<CompileEntry> {
        vec![
            CompileEntry {
                directory: PathBuf::from("/project"),
                file: PathBuf::from("/project/src/main.c"),
                arguments: vec!["gcc".into(), "-c".into(), "src/main.c".into()],
            },
            CompileEntry {
                directory: PathBuf::from("/project"),
                file: PathBuf::from("/project/src/foo.c"),
                arguments: vec!["gcc".into(), "-c".into(), "src/foo.c".into()],
            },
        ]
    }

    #[test]
    fn json_format_correct() {
        let entries = sample_entries();
        let json = serde_json::to_string_pretty(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0]["directory"], "/project");
        assert_eq!(parsed[0]["file"], "/project/src/main.c");
        assert_eq!(
            parsed[0]["arguments"],
            serde_json::json!(["gcc", "-c", "src/main.c"])
        );
        assert!(parsed[0].get("output").is_none());

        assert_eq!(parsed[1]["file"], "/project/src/foo.c");
    }

    #[test]
    fn atomic_write_cleans_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("compile_commands.json");
        let entries = sample_entries();

        write_to_json(&entries, &path).unwrap();

        assert!(path.exists(), "final file should exist");
        assert!(
            !path.with_extension("tmp").exists(),
            "temp file should be cleaned up"
        );

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn dry_run() {
        let entries = sample_entries();
        let json = serde_json::to_string_pretty(&entries).unwrap();
        assert!(json.contains("/project/src/main.c"));
        assert!(json.contains("gcc"));
    }

    #[test]
    fn auto_create_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("sub/deep/compile_commands.json");

        write_to_json(&sample_entries(), &nested).unwrap();
        assert!(nested.exists());

        let content = fs::read_to_string(&nested).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn empty_entries() {
        let entries: Vec<CompileEntry> = vec![];
        let json = serde_json::to_string_pretty(&entries).unwrap();
        assert_eq!(json, "[]");

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.json");
        write_to_json(&entries, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "[]");
    }

    #[test]
    fn print_json_does_not_panic() {
        let entries = sample_entries();
        print_json(&entries).unwrap();
    }
}
