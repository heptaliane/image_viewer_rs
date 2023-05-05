use std::{cmp::Ordering, path::Path, path::PathBuf};

use super::path::{get_child_files, next_directory, prev_directory};

#[derive(Default)]
pub struct ViewerState {
    filenames: Vec<String>,
    cursor: usize,
    pattern: String,
}

impl ViewerState {
    pub fn new(filename: String) -> Self {
        Self {
            filenames: vec![filename],
            cursor: 0,
            pattern: String::from("*"),
        }
    }

    pub fn reload_files(&mut self) -> Result<(), String> {
        match self.get() {
            Ok(filename) => match filename.parent() {
                Some(parent) => match get_child_files(parent, &self.pattern) {
                    Ok(files) if files.len() > 0 => {
                        self.filenames = files;
                        self.cursor = self.filenames
                            .iter()
                            .position(|path| {
                                path.partial_cmp(&filename.to_str().unwrap().to_string())
                                    .unwrap()
                                    == Ordering::Greater
                            })
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

    pub fn set_pattern(&mut self, pattern: String) -> Result<(), String> {
        self.pattern = pattern;
        self.reload_files()
    }

    pub fn get(&self) -> Result<PathBuf, String> {
        match self.filenames.get(self.cursor) {
            Some(filename) => match Path::new(&filename).canonicalize() {
                Ok(abspath) => Ok(abspath),
                Err(err) => Err(format!("{:?}", err)),
            },
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
        F: Fn(&Path) -> Option<String>,
    {
        match self.parent_dir() {
            Ok(parent) => {
                let mut current = parent.clone();
                loop {
                    match modifier(&current) {
                        Some(dirname) => {
                            current = PathBuf::from(&dirname);
                            match get_child_files(&current, &self.pattern) {
                                Ok(files) if files.len() > 0 => {
                                    self.filenames = files;
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
        self.change_directory(next_directory)
    }

    pub fn prev_directory(&mut self) -> Result<(), String> {
        self.change_directory(prev_directory)
    }

    pub fn set_offset(&mut self, offset: i32) -> Result<(), String> {
        let mut buffer = offset;
        loop {
            if buffer < 0 {
                match self.prev_directory() {
                    Err(err) => return Err(err),
                    _ => match self.filenames.len() as i32 {
                        n if n + buffer < 0 => buffer += n,
                        n => {
                            self.cursor = (n + buffer) as usize;
                            return Ok(());
                        }
                    },
                }
            } else if (buffer as usize) < self.filenames.len() {
                self.cursor = buffer as usize;
                return Ok(());
            } else {
                match self.next_directory() {
                    Err(err) => return Err(err),
                    _ => match self.filenames.len() as i32 {
                        n if n > buffer => buffer -= n,
                        _ => {
                            self.cursor = buffer as usize;
                            return Ok(());
                        }
                    },
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
    let expected_filenames = vec![
        "test_data/state/a/b/a",
        "test_data/state/a/b/b",
        "test_data/state/a/b/c",
    ];

    let mut state1 = ViewerState::new("test_data/state/a/b/a".to_string());
    assert_eq!(state1.reload_files(), Ok(()));
    assert_eq!(state1.filenames, expected_filenames);
    assert_eq!(state1.cursor, 0);

    let mut state2 = ViewerState::new("test_data/state/a/b/b".to_string());
    assert_eq!(state2.reload_files(), Ok(()));
    assert_eq!(expected_filenames, state2.filenames);
    assert_eq!(state2.cursor, 1);
}

#[test]
fn test_viewer_state_change_directory() {
    let mut state = ViewerState::new("test_data/state/a/b/a".to_string());
    assert_eq!(state.reload_files(), Ok(()));

    let prev_expected = vec![
        "test_data/state/a/a/a",
        "test_data/state/a/a/b",
        "test_data/state/a/a/c",
    ];
    assert_eq!(state.prev_directory(), Ok(()));
    assert_eq!(state.filenames, prev_expected);

    let next_expected = vec![
        "test_data/state/a/b/a",
        "test_data/state/a/b/b",
        "test_data/state/a/b/c",
    ];
    assert_eq!(state.next_directory(), Ok(()));
    assert_eq!(state.filenames, next_expected);
}

#[test]
fn test_viewer_state_set_offset() {
    let mut state = ViewerState::new("test_data/state/a/b/a".to_string());
    state.reload_files();
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/b/a",
    );

    assert_eq!(state.set_offset(1), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/b/b",
    );

    assert_eq!(state.set_offset(-1), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/a/c",
    );

    assert_eq!(state.set_offset(3), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/b/a",
    );
}

#[test]
fn test_viewer_state_move_cursor() {
    let mut state = ViewerState::new("test_data/state/a/b/a".to_string());
    assert_eq!(state.reload_files(), Ok(()));

    assert_eq!(state.move_cursor(1), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/b/b",
    );

    assert_eq!(state.move_cursor(2), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/d",
    );

    assert_eq!(state.move_cursor(-1), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/b/c",
    );

    assert_eq!(state.move_cursor(-10), Ok(()));
    assert_eq!(
        state.get().unwrap().to_str().unwrap(),
        "test_data/state/a/a/c",
    );
}
