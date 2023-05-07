use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use std::env::current_dir;

pub fn get_abspath(path: &Path) -> Result<PathBuf, String> {
    match path.is_relative() {
        true => match current_dir() {
            Ok(base_dir) => Ok(base_dir.join(path)),
            Err(err) => Err(format!("{:?}", err)),
        },
        false => Ok(path.to_path_buf()),
    }
}

fn get_children<F, G, T>(
    parent: &Path,
    predicate: &F,
    sort_elem: &G,
) -> Result<Vec<PathBuf>, String>
where
    F: Fn(&PathBuf) -> bool,
    G: Fn(&PathBuf) -> T,
    T: Ord,
{
    match read_dir(parent) {
        Ok(entries) => {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(predicate)
                .collect();
            paths.sort_unstable_by(|p1, p2| sort_elem(p1).partial_cmp(&sort_elem(p2)).unwrap());
            Ok(paths)
        }
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_child_files<F, T>(
    parent: &Path,
    extensions: &HashSet<String>,
    sort_elem: &F,
) -> Result<Vec<PathBuf>, String>
where
    F: Fn(&PathBuf) -> T,
    T: Ord,
{
    get_children(
        parent,
        &|path| {
            path.is_file()
                && match path.extension() {
                    Some(ext) => match ext.to_str() {
                        Some(extension) => extensions.contains(&extension.to_ascii_lowercase()),
                        _ => false,
                    },
                    _ => false,
                }
        },
        sort_elem,
    )
}

pub fn get_child_directories<F, T>(parent: &Path, sort_elem: &F) -> Result<Vec<PathBuf>, String>
where
    F: Fn(&PathBuf) -> T,
    T: Ord,
{
    get_children(parent, &|path| path.is_dir(), sort_elem)
}

pub fn next_directory<F, T>(path: &PathBuf, sort_elem: &F) -> Option<PathBuf>
where
    F: Fn(&PathBuf) -> T,
    T: Ord,
{
    if let Ok(dirs) = get_child_directories(&path, sort_elem) {
        if let Some(next_dir) = VecDeque::from(dirs).pop_front() {
            return Some(next_dir);
        }
    }

    let mut current = path.clone();
    while let Some(parent) = current.parent() {
        match get_child_directories(&parent, sort_elem) {
            Ok(mut dirs) => {
                if let Some(cursor) = dirs.iter().position(|dir| {
                    sort_elem(dir).partial_cmp(&sort_elem(path)) == Some(Ordering::Greater)
                }) {
                    return Some(dirs.remove(cursor));
                }
            }
            Err(err) => log::info!("{:?}", err),
        }
        current = parent.to_path_buf();
    }

    None
}

pub fn prev_directory<F, T>(path: &PathBuf, sort_elem: &F) -> Option<PathBuf>
where
    F: Fn(&PathBuf) -> T,
    T: Ord,
{
    if let Some(parent) = path.parent() {
        match get_child_directories(&parent, sort_elem) {
            Ok(mut dirs) => {
                dirs.reverse();
                match dirs.iter().position(|dir| {
                    sort_elem(dir).partial_cmp(&sort_elem(path)) == Some(Ordering::Less)
                }) {
                    Some(cursor) => loop {
                        let mut dir = dirs.remove(cursor);
                        while let Ok(mut children) = get_child_directories(&dir, sort_elem) {
                            if children.len() > 0 {
                                dir = children.pop().unwrap();
                            } else {
                                return Some(dir);
                            }
                        }
                        return Some(dir);
                    },
                    None => return Some(parent.to_path_buf()),
                }
            }
            Err(err) => log::info!("{:?}", err),
        }
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
    let extensions = HashSet::from([String::from("txt")]);
    let sort_elem = |path: &PathBuf| path.clone();
    let get_filenames = |paths: Vec<PathBuf>| {
        paths
            .iter()
            .map(|path| path.to_str().unwrap().to_string())
            .collect::<Vec<String>>()
    };

    let parent1 = Path::new("test_data/state/a/a");
    let actual1 = get_child_files(parent1, &extensions, &sort_elem);
    assert!(actual1.is_ok());
    assert_eq!(
        get_filenames(actual1.unwrap()),
        vec![
            "test_data/state/a/a/a.txt".to_string(),
            "test_data/state/a/a/b.txt".to_string(),
            "test_data/state/a/a/c.txt".to_string(),
        ]
    );

    let parent2 = Path::new("test_data/state/a");
    let actual2 = get_child_files(parent2, &extensions, &sort_elem);
    assert!(actual2.is_ok());
    assert_eq!(
        get_filenames(actual2.unwrap()),
        vec!["test_data/state/a/d.txt".to_string()]
    );

    let parent3 = Path::new("test_data/state/a/c");
    let actual3 = get_child_files(parent3, &extensions, &sort_elem);
    let expected3: Vec<String> = vec![];
    assert!(actual3.is_ok());
    assert_eq!(get_filenames(actual3.unwrap()), expected3);
}

#[test]
fn test_get_directories() {
    let sort_elem = |path: &PathBuf| path.clone();
    let get_filenames = |paths: Vec<PathBuf>| {
        paths
            .iter()
            .map(|path| path.to_str().unwrap().to_string())
            .collect::<Vec<String>>()
    };

    let parent1 = Path::new("test_data/state");
    let actual1 = get_child_directories(parent1, &sort_elem);
    assert!(actual1.is_ok());
    assert_eq!(
        get_filenames(actual1.unwrap()),
        vec![
            "test_data/state/a".to_string(),
            "test_data/state/b".to_string(),
            "test_data/state/c".to_string(),
        ]
    );

    let parent2 = Path::new("test_data/state/a/c");
    let actual2 = get_child_directories(parent2, &sort_elem);
    let expected2: Vec<String> = vec![];
    assert!(actual2.is_ok());
    assert_eq!(get_filenames(actual2.unwrap()), expected2);
}

#[test]
fn test_next_directory() {
    let sort_elem = |path: &PathBuf| path.clone();

    let path1 = Path::new("test_data/state/a");
    let expected1 = "test_data/state/a/a";
    let actual1 = next_directory(&path1.to_path_buf(), &sort_elem);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap().to_str().unwrap(), expected1);

    let path2 = Path::new("test_data/state/a/b");
    let expected2 = "test_data/state/a/c";
    let actual2 = next_directory(&path2.to_path_buf(), &sort_elem);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap().to_str().unwrap(), expected2);

    let path3 = Path::new("test_data/state/a/c");
    let expected3 = "test_data/state/b";
    let actual3 = next_directory(&path3.to_path_buf(), &sort_elem);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap().to_str().unwrap(), expected3);
}

#[test]
fn test_prev_directory() {
    let sort_elem = |path: &PathBuf| path.clone();

    let path1 = Path::new("test_data/state/a/a");
    let expected1 = "test_data/state/a";
    let actual1 = prev_directory(&path1.to_path_buf(), &sort_elem);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap().to_str().unwrap(), expected1);

    let path2 = Path::new("test_data/state/a/c");
    let expected2 = "test_data/state/a/b";
    let actual2 = prev_directory(&path2.to_path_buf(), &sort_elem);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap().to_str().unwrap(), expected2);

    let path3 = Path::new("test_data/state/b");
    let expected3 = "test_data/state/a/c";
    let actual3 = prev_directory(&path3.to_path_buf(), &sort_elem);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap().to_str().unwrap(), expected3);
}
