use std::{cmp::Ordering, collections::HashSet, path::Path, path::PathBuf};

use super::path::{get_child_files, next_directory, prev_directory};

fn sort_by_path(path: &PathBuf) -> PathBuf {
    path.clone()
}

#[derive(Default)]
pub struct ViewerState {
    paths: Vec<PathBuf>,
    cursor: usize,
    extensions: HashSet<String>,
}

impl ViewerState {
    pub fn new(filename: &str, extensions: HashSet<String>) -> Self {
        Self {
            paths: vec![Path::new(filename).to_path_buf()],
            cursor: 0,
            extensions,
        }
    }

    pub fn reload_files(&mut self) -> Result<(), String> {
        match self.get() {
            Ok(current) => match current.parent() {
                Some(parent) => match get_child_files(parent, &self.extensions, &sort_by_path) {
                    Ok(paths) if paths.len() > 0 => {
                        self.paths = paths;
                        self.cursor = self
                            .paths
                            .iter()
                            .position(|path| path.partial_cmp(&current).unwrap() != Ordering::Less)
                            .unwrap_or(0);
                        Ok(())
                    }
                    Err(err) => Err(err),
                    _ => self.prev_directory(),
                },
                None => Err("Parent directory is not found.".to_string()),
            },
            Err(err) => Err(err),
        }
    }

    pub fn get(&self) -> Result<PathBuf, String> {
        match self.paths.get(self.cursor) {
            Some(path) => Ok(path.clone()),
            _ => Err(String::from("Files are not in buffer.")),
        }
    }

    fn parent_dir(&self) -> Result<PathBuf, String> {
        match self.get() {
            Ok(filename) => match filename.parent() {
                Some(parent) => Ok(parent.to_path_buf()),
                _ => Err(String::from("No parent directory is found.")),
            },
            Err(err) => Err(format!("{:?}", err)),
        }
    }

    fn change_directory<F>(&mut self, modifier: F) -> Result<(), String>
    where
        F: Fn(&PathBuf) -> Option<PathBuf>,
    {
        match self.parent_dir() {
            Ok(parent) => {
                let mut current = parent.clone();
                loop {
                    match modifier(&current) {
                        Some(dirname) => {
                            current = PathBuf::from(&dirname);
                            match get_child_files(&current, &self.extensions, &sort_by_path) {
                                Ok(paths) if paths.len() > 0 => {
                                    self.paths = paths;
                                    self.cursor = 0;
                                    return Ok(());
                                }
                                Err(err) => return Err(err),
                                _ => (),
                            }
                        }
                        None => return Err(format!("(current: {:?})", parent)),
                    }
                }
            }
            Err(err) => return Err(err),
        }
    }

    pub fn next_directory(&mut self) -> Result<(), String> {
        self.change_directory(|p| next_directory(p, &sort_by_path))
    }

    pub fn prev_directory(&mut self) -> Result<(), String> {
        self.change_directory(|p| prev_directory(p, &sort_by_path))
    }

    pub fn set_offset(&mut self, offset: i32) -> Result<(), String> {
        let mut buffer = offset;
        loop {
            if buffer < 0 {
                match self.prev_directory() {
                    Err(err) => return Err(err),
                    _ => match self.paths.len() as i32 {
                        n if n + buffer < 0 => buffer += n,
                        n => {
                            self.cursor = (n + buffer) as usize;
                            return Ok(());
                        }
                    },
                }
            } else if (buffer as usize) < self.paths.len() {
                self.cursor = buffer as usize;
                return Ok(());
            } else {
                buffer -= self.paths.len() as i32;
                match self.next_directory() {
                    Err(err) => return Err(err),
                    _ => (),
                }
            }
        }
    }

    pub fn move_cursor(&mut self, moves: i32) -> Result<(), String> {
        self.set_offset((self.cursor as i32) + moves)
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
fn test_viewer_state_init() {
    let extensions = HashSet::from([String::from("txt")]);
    let expected_filenames = [
        "test_data/state/a/b/a.txt",
        "test_data/state/a/b/b.txt",
        "test_data/state/a/b/c.txt",
    ];

    let mut state1 = ViewerState::new("test_data/state/a/b/a.txt", extensions.clone());
    assert_eq!(state1.reload_files(), Ok(()));
    assert_eq!(state1.cursor, 0);
    for (actual, expected) in state1.paths.iter().zip(expected_filenames) {
        assert!(actual.ends_with(expected));
    }

    let mut state2 = ViewerState::new("test_data/state/a/b/b.txt", extensions.clone());
    assert_eq!(state2.reload_files(), Ok(()));
    assert_eq!(state2.cursor, 1);
    for (actual, expected) in state2.paths.iter().zip(expected_filenames) {
        assert!(actual.ends_with(expected));
    }
}

#[test]
fn test_viewer_state_change_directory() {
    let extensions = HashSet::from([String::from("txt")]);
    let mut state = ViewerState::new("test_data/state/a/b/a.txt", extensions);
    assert_eq!(state.reload_files(), Ok(()));

    let prev_expected = [
        "test_data/state/a/a/a.txt",
        "test_data/state/a/a/b.txt",
        "test_data/state/a/a/c.txt",
    ];
    assert_eq!(state.prev_directory(), Ok(()));
    for (actual, expected) in state.paths.iter().zip(prev_expected) {
        assert!(actual.ends_with(expected));
    }

    let next_expected = [
        "test_data/state/a/b/a.txt",
        "test_data/state/a/b/b.txt",
        "test_data/state/a/b/c.txt",
    ];
    assert_eq!(state.next_directory(), Ok(()));
    for (actual, expected) in state.paths.iter().zip(next_expected) {
        assert!(actual.ends_with(expected));
    }
}

#[test]
fn test_viewer_state_set_offset() {
    let extensions = HashSet::from([String::from("txt")]);
    let mut state = ViewerState::new("test_data/state/a/b/a.txt", extensions);
    assert_eq!(state.reload_files(), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/b/a.txt"));

    assert_eq!(state.set_offset(1), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/b/b.txt"));

    assert_eq!(state.set_offset(-1), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/a/c.txt"));

    assert_eq!(state.set_offset(3), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/b/a.txt"));
}

#[test]
fn test_viewer_state_move_cursor() {
    let extensions = HashSet::from([String::from("txt")]);
    let mut state = ViewerState::new("test_data/state/a/b/a.txt", extensions);
    assert_eq!(state.reload_files(), Ok(()));

    assert_eq!(state.move_cursor(1), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/b/b.txt"));

    assert_eq!(state.move_cursor(2), Ok(()));
    println!("{:?}", state.get().unwrap());
    assert!(state.get().unwrap().ends_with("test_data/state/b/a/a.txt"));

    assert_eq!(state.move_cursor(-1), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/b/c.txt"));

    assert_eq!(state.move_cursor(-6), Ok(()));
    assert!(state.get().unwrap().ends_with("test_data/state/a/d.txt"));
}
