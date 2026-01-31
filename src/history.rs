use std::io::{self, ErrorKind, BufReader, BufWriter};
use std::path::Path;
use std::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionHistory {
    stack: Vec<String>,
    head: Option<usize>,
}

pub enum LoadError {
    FileNotFound,
    CannotOpenFile(io::Error),
    CannotDeserialize(serde_json::Error),
}

pub enum SaveError {
    CannotCreateFile(io::Error),
    CannotSerialize(serde_json::Error),
}

impl SessionHistory {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let file = File::open(path)
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound => LoadError::FileNotFound,
                _ => LoadError::CannotOpenFile(err),
            })?;

        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|err| LoadError::CannotDeserialize(err))
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), SaveError> {
        let file = File::create(&path)
            .map_err(|err| SaveError::CannotCreateFile(err))?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .map_err(|err| SaveError::CannotSerialize(err))
    }

    pub fn prev(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index > 0 => Some(index - 1),
            Some(_index) => None,
            None if length > 0 => panic!("no session head in a non-empty stack"),
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    pub fn next(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index < length - 1 => Some(index + 1),
            Some(_index) => None,
            None if length > 0 => panic!("no session head in a non-empty stack"),
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    fn truncate(&mut self) {
        if let Some(index) = self.head {
            while self.stack.len() > index {
                self.stack.pop();
            }

            self.head = None;
        }
    }

    pub fn add_session(&mut self, session: String) {
        let length = self.stack.len();
        match self.head {
            // move the head if the next session in the stack is the one we
            // wanted to add
            Some(index) if index < length - 1 && self.stack[index + 1] == session => {
                self.head = Some(index + 1);
            },
            _ => {
                self.truncate();
                self.stack.push(session);
            }
        }
    }
}
