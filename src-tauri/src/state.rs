use std::path::{Path, PathBuf};

enum GetNextDirectoryResult {
    DirectoryFound(String),
    ParentNotFound,
    AccessDenied,
    NoBrotherLeft,
}

fn get_children<F>(parent: &Path, predicate: F) -> Result<Vec<String>, ()>
where
    F: Fn(&PathBuf) -> bool,
{
    match parent.read_dir() {
        Ok(dirs) => {
            let mut directories: Vec<String> = dirs
                .filter(|path| match path {
                    Ok(_) => true,
                    _ => false,
                })
                .map(|path| path.unwrap().path())
                .filter(predicate)
                .map(|path| path.to_str().unwrap().to_string())
                .collect();
            directories.sort();
            Ok(directories)
        }
        _ => Err(()),
    }
}

fn get_child_dirs(parent: &Path) -> Result<Vec<String>, ()> {
    get_children(parent, |path| path.is_dir())
}

fn get_child_files(parent: &Path) -> Result<Vec<String>, ()> {
    get_children(parent, |path| path.is_file())
}

fn get_deep_child_dir(parent: &Path, head: bool) -> Option<String> {
    let mut current = parent.to_str().unwrap().to_string();

    loop {
        if !head {
            let result = get_child_files(&Path::new(&current));
            match result {
                Ok(files) => {
                    if files.len() > 0 {
                        return Some(current);
                    }
                }
                _ => return None,
            }
        }

        let result = get_child_dirs(&Path::new(&current));
        match result {
            Ok(dirs) => {
                if dirs.len() == 0 {
                    return Some(current);
                }

                current = match head {
                    true => dirs.first().unwrap().clone(),
                    false => dirs.last().unwrap().clone(),
                };
            }
            _ => return None,
        }
    }
}

fn get_next_directory(dirpath: &String, increase: bool) -> GetNextDirectoryResult {
    match Path::new(dirpath).parent() {
        Some(parent) => match get_child_dirs(&parent) {
            Ok(dirs) => {
                let idx = dirs.iter().position(|path| path == dirpath).unwrap();

                if increase {
                    match dirs.get(idx + 1) {
                        Some(dir) => GetNextDirectoryResult::DirectoryFound(dir.clone()),
                        None => GetNextDirectoryResult::NoBrotherLeft,
                    }
                } else {
                    match idx {
                        0 => GetNextDirectoryResult::NoBrotherLeft,
                        _ => {
                            let next_idx = ((idx as i32) - 1) as usize;
                            GetNextDirectoryResult::DirectoryFound(dirs[next_idx].clone())
                        }
                    }
                }
            }
            _ => GetNextDirectoryResult::AccessDenied,
        },
        None => GetNextDirectoryResult::ParentNotFound,
    }
}

fn change_directory(dirpath: &String, increase: bool) -> Option<String> {
    let mut current = dirpath.clone();

    if !increase {
        if let Ok(children) = get_child_dirs(&Path::new(&current)) {
            if children.len() > 0 {
                current = children.last().unwrap().clone();
            }
        }
    }

    loop {
        match get_next_directory(&current, increase) {
            GetNextDirectoryResult::ParentNotFound => return None,
            GetNextDirectoryResult::AccessDenied => return None,
            GetNextDirectoryResult::DirectoryFound(dir) => {
                if let Some(dirname) = get_deep_child_dir(Path::new(&dir), increase) {
                    match get_child_files(Path::new(&dirname)) {
                        Ok(files) => {
                            current = dirname;
                            if files.len() > 0 {
                                return Some(current);
                            }
                        }
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            GetNextDirectoryResult::NoBrotherLeft if increase => {
                let parent = Path::new(&current).parent().unwrap();
                let result = get_child_files(&parent);
                current = parent.to_str().unwrap().to_string();

                match result {
                    Ok(files) => {
                        if files.len() > 0 {
                            return Some(current);
                        }
                    }
                    _ => return None,
                }
            }
            GetNextDirectoryResult::NoBrotherLeft => match Path::new(&current).parent() {
                Some(parent) => {
                    current = parent.to_str().unwrap().to_string();
                }
                _ => return None,
            },
        }
    }
}

#[derive(Default)]
pub struct ViewerState {
    pub filenames: Vec<String>,
    pub cursor: usize,
}

impl ViewerState {
    pub fn new(filename: &String) -> Self {
        let parent = Path::new(filename).parent().unwrap();
        let filenames = get_child_files(&parent).unwrap();
        Self {
            filenames: filenames.clone(),
            cursor: filenames.iter().position(|path| path == filename).unwrap(),
        }
    }

    pub fn get(&self) -> Option<&String> {
        self.filenames.get(self.cursor)
    }

    pub fn set_offset(&mut self, offset: i32) -> Result<(), ()> {
        match offset {
            i if i < 0 => self.prev_directory(),
            i if (i as usize) >= self.filenames.len() => self.next_directory(),
            i => {
                self.cursor = i as usize;
                Ok(())
            }
        }
    }

    pub fn prev_directory(&mut self) -> Result<(), ()> {
        match change_directory(&self.current_dir(), false) {
            Some(prev_dir) => match get_child_files(&Path::new(&prev_dir)) {
                Ok(files) => {
                    self.filenames = files;
                    self.cursor = self.filenames.len() - 1;
                    Ok(())
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    pub fn next_directory(&mut self) -> Result<(), ()> {
        match change_directory(&self.current_dir(), true) {
            Some(next_dir) => match get_child_files(&Path::new(&next_dir)) {
                Ok(files) => {
                    self.filenames = files;
                    self.cursor = 0;
                    Ok(())
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    pub fn move_cursor(&mut self, moves: i32) -> Result<(), ()> {
        self.set_offset((self.cursor as i32) + moves)
    }

    fn current_dir(&self) -> String {
        let filename = self.filenames.first().unwrap();
        let current_dir = Path::new(filename).parent().unwrap();
        return current_dir.to_str().unwrap().to_string();
    }
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
fn test_get_child_dirs() {
    let parent1 = Path::new("test_data/state/a/");
    let dirs1 = get_child_dirs(&parent1);
    assert_eq!(
        vec![
            "test_data/state/a/a",
            "test_data/state/a/b",
            "test_data/state/a/c"
        ],
        dirs1.expect("get_child_dirs failed")
    );

    let parent2 = Path::new("test_data/state/c");
    let dirs2 = get_child_dirs(&parent2);
    let expect2: Vec<String> = vec![];
    assert_eq!(expect2, dirs2.expect("get_child_dirs failed"));
}

#[test]
fn test_get_child_files() {
    let parent1 = Path::new("test_data/state/a/a/");
    let files1 = get_child_files(&parent1);
    assert_eq!(
        vec![
            "test_data/state/a/a/a",
            "test_data/state/a/a/b",
            "test_data/state/a/a/c"
        ],
        files1.expect("get_child_files failed")
    );

    let parent2 = Path::new("test_data/state/a/");
    let files2 = get_child_files(&parent2);
    assert_eq!(
        vec!["test_data/state/a/d"],
        files2.expect("get_child_files failed")
    );
}

#[test]
fn test_get_deep_child_dir() {
    let parent1 = Path::new("test_data/state/a");
    let dirname1a = get_deep_child_dir(parent1, true);
    let dirname1b = get_deep_child_dir(parent1, false);
    assert_eq!("test_data/state/a/a", dirname1a.unwrap());
    assert_eq!("test_data/state/a", dirname1b.unwrap());

    let parent2 = Path::new("test_data/state");
    let dirname2a = get_deep_child_dir(parent2, true);
    let dirname2b = get_deep_child_dir(parent2, false);
    assert_eq!("test_data/state/a/a", dirname2a.unwrap());
    assert_eq!("test_data/state/c", dirname2b.unwrap());
}

#[test]
fn test_change_directory() {
    let directories = [
        "test_data/state/a/a",
        "test_data/state/a/b",
        "test_data/state/a",
        "test_data/state/b",
        "test_data/state/c",
    ];
    let expects = [
        (Some("test_data/state/a/b"), None),
        (Some("test_data/state/a"), Some("test_data/state/a/a")),
        (Some("test_data/state/b/a"), Some("test_data/state/a/b")),
        (Some("test_data/state/c"), Some("test_data/state/b/b")),
        (None, Some("test_data/state/b/b")),
    ];

    for (dir, (expect1, expect2)) in directories.iter().zip(expects.iter()) {
        let actual1 = change_directory(&dir.to_string(), true);
        let actual2 = change_directory(&dir.to_string(), false);
        assert_eq!(
            match expect1 {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            actual1
        );
        assert_eq!(
            match expect2 {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            actual2
        );
    }
}

#[test]
fn test_viewer_state_init() {
    let state1 = ViewerState::new(&"test_data/state/a/b/a".to_string());
    let expected_filenames = vec![
        "test_data/state/a/b/a",
        "test_data/state/a/b/b",
        "test_data/state/a/b/c",
    ];
    assert_eq!(expected_filenames, state1.filenames);
    assert_eq!(0, state1.cursor);

    let state2 = ViewerState::new(&"test_data/state/a/b/b".to_string());
    assert_eq!(expected_filenames, state2.filenames);
    assert_eq!(1, state2.cursor);
}

#[test]
fn test_viewer_state_change_directory() {
    let mut state = ViewerState::new(&"test_data/state/a/b/a".to_string());

    let prev_expected = vec![
        "test_data/state/a/a/a",
        "test_data/state/a/a/b",
        "test_data/state/a/a/c",
    ];
    state.prev_directory();
    assert_eq!(prev_expected, state.filenames);

    let next_expected = vec![
        "test_data/state/a/b/a",
        "test_data/state/a/b/b",
        "test_data/state/a/b/c",
    ];
    state.next_directory();
    assert_eq!(next_expected, state.filenames);
}

#[test]
fn test_viewer_state_set_offset() {
    let mut state = ViewerState::new(&"test_data/state/a/b/a".to_string());
    assert_eq!("test_data/state/a/b/a", state.get().unwrap());

    state.set_offset(1);
    assert_eq!("test_data/state/a/b/b", state.get().unwrap());

    state.set_offset(-1);
    assert_eq!("test_data/state/a/a/c", state.get().unwrap());

    state.set_offset(3);
    assert_eq!("test_data/state/a/b/a", state.get().unwrap());
}

#[test]
fn test_viewer_state_move_cursor() {
    let mut state = ViewerState::new(&"test_data/state/a/b/a".to_string());
    state.move_cursor(1);
    assert_eq!("test_data/state/a/b/b", state.get().unwrap());

    state.move_cursor(2);
    assert_eq!("test_data/state/a/d", state.get().unwrap());

    state.move_cursor(-1);
    assert_eq!("test_data/state/a/b/c", state.get().unwrap());

    state.move_cursor(-10);
    assert_eq!("test_data/state/a/a/c", state.get().unwrap());
}
