use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
        log::debug!("Initialized state with {:?}", filename);
        log::info!("Available extensions: {:?}", extensions);

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
                        log::debug!(
                            "File list updated ({:?} files, cursor = {:?})",
                            self.paths.len(),
                            self.cursor
                        );
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
                    log::debug!("Changing directory (current target: {:?})", current);
                    match modifier(&current) {
                        Some(dirname) => {
                            current = PathBuf::from(&dirname);
                            match get_child_files(&current, &self.extensions, &sort_by_path) {
                                Ok(paths) if paths.len() > 0 => {
                                    self.paths = paths;
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
        match self.change_directory(|p| next_directory(p, &sort_by_path)) {
            Ok(_) => {
                self.move_first();
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn prev_directory(&mut self) -> Result<(), String> {
        match self.change_directory(|p| prev_directory(p, &sort_by_path)) {
            Ok(_) => {
                self.move_last();
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn next_cursor(&mut self) -> Result<(), String> {
        match self.cursor + 1 < self.paths.len() {
            true => {
                self.cursor += 1;
                Ok(())
            }
            false => self.next_directory(),
        }
    }

    pub fn prev_cursor(&mut self) -> Result<(), String> {
        match self.cursor > 0 {
            true => {
                self.cursor -= 1;
                Ok(())
            }
            false => self.prev_directory(),
        }
    }

    pub fn move_first(&mut self) {
        self.cursor = 0;
    }

    pub fn move_last(&mut self) {
        self.cursor = self.paths.len() - 1;
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
fn test_viewer_state_next_cursor() {
    let extensions = HashSet::from([String::from("txt")]);
    let mut state = ViewerState::new("test_data/state/a/a/b.txt", extensions);
    assert_eq!(state.reload_files(), Ok(()));

    assert!(state.next_cursor().is_ok());
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/a/a/c.txt").to_path_buf()
    );

    assert!(state.next_cursor().is_ok());
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/a/b/a.txt").to_path_buf()
    );

    assert!(state.next_cursor().is_ok());  // a/b/b
    assert!(state.next_cursor().is_ok());  // a/b/c
    assert!(state.next_cursor().is_ok());  // b/a/a
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/b/a/a.txt").to_path_buf()
    );
}

#[test]
fn test_viewer_state_prev_cursor() {
    let extensions = HashSet::from([String::from("txt")]);
    let mut state = ViewerState::new("test_data/state/b/a/a.txt", extensions);
    assert_eq!(state.reload_files(), Ok(()));

    assert!(state.prev_cursor().is_ok());
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/a/b/c.txt").to_path_buf()
    );

    assert!(state.prev_cursor().is_ok());
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/a/b/b.txt").to_path_buf()
    );

    assert!(state.prev_cursor().is_ok());  // a/b/a
    assert!(state.prev_cursor().is_ok());  // a/a/c
    assert!(state.prev_cursor().is_ok());  // a/a/b
    assert!(state.prev_cursor().is_ok());  // a/a/a
    assert!(state.prev_cursor().is_ok());  // a/d
    assert_eq!(
        state.get().unwrap(),
        Path::new("test_data/state/a/d.txt").to_path_buf()
    );
}

#[test]
fn test_viewer_state_move_first() {
    let extensions = HashSet::from([String::from("txt")]);

    let mut state1 = ViewerState::new("test_data/state/a/b/a.txt", extensions.clone());
    assert_eq!(state1.reload_files(), Ok(()));

    state1.move_first();
    assert_eq!(
        state1.get(),
        Ok(Path::new("test_data/state/a/b/a.txt").to_path_buf())
    );

    let mut state2 = ViewerState::new("test_data/state/a/b/b.txt", extensions.clone());
    assert_eq!(state2.reload_files(), Ok(()));

    state2.move_first();
    assert_eq!(
        state2.get(),
        Ok(Path::new("test_data/state/a/b/a.txt").to_path_buf())
    );

    let mut state3 = ViewerState::new("test_data/state/a/b/c.txt", extensions.clone());
    assert_eq!(state3.reload_files(), Ok(()));

    state3.move_first();
    assert_eq!(
        state3.get(),
        Ok(Path::new("test_data/state/a/b/a.txt").to_path_buf())
    );
}

#[test]
fn test_viewer_state_move_last() {
    let extensions = HashSet::from([String::from("txt")]);

    let mut state1 = ViewerState::new("test_data/state/a/a/a.txt", extensions.clone());
    assert_eq!(state1.reload_files(), Ok(()));

    state1.move_last();
    assert_eq!(
        state1.get(),
        Ok(Path::new("test_data/state/a/a/c.txt").to_path_buf())
    );

    let mut state2 = ViewerState::new("test_data/state/a/a/b.txt", extensions.clone());
    assert_eq!(state2.reload_files(), Ok(()));

    state2.move_last();
    assert_eq!(
        state2.get(),
        Ok(Path::new("test_data/state/a/a/c.txt").to_path_buf())
    );

    let mut state3 = ViewerState::new("test_data/state/a/a/c.txt", extensions.clone());
    assert_eq!(state3.reload_files(), Ok(()));

    state3.move_last();
    assert_eq!(
        state3.get(),
        Ok(Path::new("test_data/state/a/a/c.txt").to_path_buf())
    );
}
