use std::cmp::Ordering;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use glob::glob;

pub fn get_abspath(path: &Path) -> Result<PathBuf, String> {
    match path.is_relative() {
        true => match current_dir() {
            Ok(base_dir) => Ok(base_dir.join(path)),
            Err(err) => Err(format!("{:?}", err)),
        },
        false => Ok(path.to_path_buf()),
    }
}

fn get_children<F>(parent: &Path, pattern: &str, predicate: F) -> Result<Vec<String>, String>
where
    F: Fn(&PathBuf) -> bool,
{
    match glob(parent.join(pattern).to_str().unwrap()) {
        Ok(paths) => {
            let mut children: Vec<String> = paths
                .filter_map(|path| path.ok())
                .filter(predicate)
                .filter_map(|path| match path.to_str() {
                    Some(value) => Some(value.to_string()),
                    _ => None,
                })
                .collect();
            children.sort();
            Ok(children)
        }
        Err(err) => Err(err.to_string()),
    }
}

fn get_child_directories(parent: &Path, recursive: bool) -> Result<Vec<String>, String> {
    get_children(
        parent,
        match recursive {
            true => "**",
            false => "*",
        },
        |path| path.is_dir(),
    )
}

pub fn get_child_files(parent: &Path, pattern: &str) -> Result<Vec<String>, String> {
    get_children(parent, pattern, |path| path.is_file())
}

pub fn next_directory(path: &Path) -> Option<String> {
    let children = get_child_directories(path, true).unwrap();
    if let Some(child) = children.first() {
        return Some(child.to_owned());
    }

    let mut current = path.to_str().unwrap().to_string();
    while let Some(parent) = Path::new(&current).parent() {
        let children = get_child_directories(parent, false).unwrap();
        if let Some(cursor) = children
            .iter()
            .position(|child| child.partial_cmp(&current).unwrap() == Ordering::Greater)
        {
            return Some(children[cursor].clone());
        }
        current = parent.to_str().unwrap().to_string();
    }
    None
}

pub fn prev_directory(path: &Path) -> Option<String> {
    let current = path.to_str().unwrap().to_string();
    if let Some(parent) = Path::new(path).parent() {
        let mut children = get_child_directories(parent, true).unwrap();
        children.reverse();
        return match children
            .iter()
            .position(|child| child.partial_cmp(&current).unwrap() == Ordering::Less)
        {
            Some(cursor) => Some(children[cursor].clone()),
            None => Some(parent.to_str().unwrap().to_string()),
        };
    }
    None
}

/*
 * Test data structure:
 * test_data/state/ +- a/ +- a/ +- a
 *                  |     |     +- b
 *                  |     |     +- c
 *                  |     |
 *                  |     +- b/ +- a
 *                  |     |     +- b
 *                  |     |     +- c
 *                  |     +- c/
 *                  |     +- d
 *                  |
 *                  +- b/ +- a/ +- a
 *                  |     |
 *                  |     +- b/ +- a
 *                  |     |
 *                  |     +- c/
 *                  |
 *                  +- c/ +- a
 *                        +- b
 *                        +- c
 */
#[test]
fn test_get_child_files() {
    let pattern = "[!.]*";

    let parent1 = Path::new("test_data/state/a/a");
    let actual1 = get_child_files(parent1, pattern);
    assert_eq!(
        actual1,
        Ok(vec![
            "test_data/state/a/a/a".to_string(),
            "test_data/state/a/a/b".to_string(),
            "test_data/state/a/a/c".to_string(),
        ])
    );

    let parent2 = Path::new("test_data/state/a");
    assert_eq!(
        get_child_files(parent2, pattern),
        Ok(vec!["test_data/state/a/d".to_string()])
    );

    let parent3 = Path::new("test_data/state/a/c");
    assert_eq!(get_child_files(parent3, pattern), Ok(vec![]));
}

#[test]
fn test_get_directories() {
    let parent1 = Path::new("test_data/state");
    let actual1a = get_child_directories(parent1, true);
    assert_eq!(
        actual1a,
        Ok(vec![
            "test_data/state/a".to_string(),
            "test_data/state/a/a".to_string(),
            "test_data/state/a/b".to_string(),
            "test_data/state/a/c".to_string(),
            "test_data/state/b".to_string(),
            "test_data/state/b/a".to_string(),
            "test_data/state/b/b".to_string(),
            "test_data/state/b/c".to_string(),
            "test_data/state/c".to_string(),
        ])
    );
    let actual1b = get_child_directories(parent1, false);
    assert_eq!(
        actual1b,
        Ok(vec![
            "test_data/state/a".to_string(),
            "test_data/state/b".to_string(),
            "test_data/state/c".to_string(),
        ])
    );

    let parent2 = Path::new("test_data/state/a/c");
    assert_eq!(get_child_directories(parent2, true), Ok(vec![]));
}

#[test]
fn test_next_directory() {
    let path1 = Path::new("test_data/state/a");
    let expected1 = Some("test_data/state/a/a".to_string());
    assert_eq!(next_directory(&path1), expected1);

    let path2 = Path::new("test_data/state/a/a");
    let expected2 = Some("test_data/state/a/b".to_string());
    assert_eq!(next_directory(&path2), expected2);

    let path3 = Path::new("test_data/state/a/c");
    let expected3 = Some("test_data/state/b".to_string());
    assert_eq!(next_directory(&path3), expected3);
}

#[test]
fn test_prev_directory() {
    let path1 = Path::new("test_data/state/a/a");
    let expected1 = Some("test_data/state/a".to_string());
    assert_eq!(prev_directory(&path1), expected1);

    let path2 = Path::new("test_data/state/a/b");
    let expected2 = Some("test_data/state/a/a".to_string());
    assert_eq!(prev_directory(&path2), expected2);

    let path3 = Path::new("test_data/state/b");
    let expected3 = Some("test_data/state/a/c".to_string());
    assert_eq!(prev_directory(&path3), expected3);
}
