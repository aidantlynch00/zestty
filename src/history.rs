use std::io::{self, ErrorKind, BufReader, BufWriter};
use std::path::Path;
use std::fs::File;
use std::collections::HashSet;
use zellij_tile::prelude::SessionInfo;
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

static NO_HEAD: &'static str = "no session head in a non-empty stack";

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

    #[tracing::instrument(skip(sessions))]
    pub fn remove_dead_sessions(&mut self, sessions: &Vec<SessionInfo>) {
        if self.stack.len() == 0 {
            return;
        }

        let mut session_set = HashSet::<&str>::new();
        for session in sessions {
            session_set.insert(&session.name);
        }

        tracing::debug!("active sessions: {:?}", session_set);

        let head_index = self.head.expect(NO_HEAD);
        let mut removed_before_head: usize = 0;
        let old_stack = std::mem::take(&mut self.stack);

        for (index, session) in old_stack.into_iter().enumerate() {
            if session_set.contains(session.as_str()) {
                self.stack.push(session);
            }
            else if index < head_index {
                removed_before_head += 1;
            }
        }

        if self.stack.len() == 0 {
            self.head = None;
        }
        else {
            // SAFETY: subtraction will never underflow
            self.head = Some(head_index - removed_before_head);
        }
    }

    pub fn prev(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index > 0 => Some(index - 1),
            Some(_index) => None,
            None if length > 0 => panic!("{}", NO_HEAD),
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
            None if length > 0 => panic!("{}", NO_HEAD),
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    // TODO: if session is present in stack already, should we remove dupes?
    // does this change if dupes are ahead or behind head?
    #[tracing::instrument]
    pub fn add_session(&mut self, session: String) {
        let length = self.stack.len();
        match self.head {
            // do nothing if the added session is already at head
            Some(index) if index < length && self.stack[index] == session => {
                tracing::debug!("session already at head");
            },
            // move the head if the next session in the stack is the one we
            // wanted to add
            Some(index) if index < length - 1 && self.stack[index + 1] == session => {
                tracing::debug!("session next in history stack");
                self.head = Some(index + 1);
            },
            Some(index) => {
                tracing::debug!("truncating history stack");
                self.stack.truncate(index + 1);
                self.stack.push(session);
                self.head = Some(index + 1);
            },
            None if length > 0 => panic!("{}", NO_HEAD),
            None => {
                tracing::debug!("starting history stack");
                self.stack.push(session);
                self.head = Some(0);
            }
        }
    }
}
