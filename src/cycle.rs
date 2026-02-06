use std::collections::{VecDeque, HashSet};
use std::io::{self, ErrorKind, BufReader, BufWriter};
use std::path::Path;
use std::fs::File;
use std::fmt::Debug;
use zellij_tile::prelude::SessionInfo;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionCycle {
    sessions: VecDeque<String>,
    prev: Option<String>,
    curr: Option<String>,
}

pub enum LoadError {
    FileNotFound,
    CouldNotOpenFile(io::Error),
    CouldNotDeserialize(serde_json::Error),
}

pub enum SaveError {
    CouldNotCreateFile(io::Error),
    CouldNotSerialize(serde_json::Error),
}

impl SessionCycle {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let file = File::open(path)
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound => LoadError::FileNotFound,
                _ => LoadError::CouldNotOpenFile(err),
            })?;

        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|err| LoadError::CouldNotDeserialize(err))
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), SaveError> {
        let file = File::create(&path)
            .map_err(|err| SaveError::CouldNotCreateFile(err))?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)
            .map_err(|err| SaveError::CouldNotSerialize(err))
    }

    #[tracing::instrument(skip(sessions))]
    pub fn remove_dead_sessions(&mut self, sessions: &Vec<SessionInfo>) {
        let session_set = sessions.iter()
            .map(|sessions| sessions.name.as_str())
            .collect::<HashSet<&str>>();

        tracing::debug!("active sessions: {:?}", session_set);
        let is_active = |session: &String| session_set.contains(session.as_str());

        self.sessions.retain(is_active);
        self.prev.take_if(|s| !is_active(s));
        self.curr.take_if(|s| !is_active(s));
    }

    fn update_curr(&mut self) {
        let session = self.sessions.front().map(String::clone);
        self.prev = self.curr.take();
        self.curr = session;
    }

    fn front_to_back(&mut self) {
        match self.sessions.pop_front() {
            Some(front) => self.sessions.push_back(front),
            None => { }
        }
    }

    fn cycle_to_session(&mut self, session: &String) {
        if self.sessions.len() == 0 {
            return;
        }

        // SAFETY: sessions contains at least one element
        let mut front = self.sessions.front().unwrap();
        let start = front.clone();
        loop {
            if *front == *session {
                return;
            }

            self.front_to_back();

            // SAFETY: sessions contains at least one element
            front = self.sessions.front().unwrap();
            if *front == *start {
                return;
            }
        }
    }

    pub fn back(&mut self) -> Option<&str> {
        let prev = self.prev.take()?;
        self.prev = self.curr.take();

        self.cycle_to_session(&prev);
        self.curr = Some(prev);
        self.curr.as_ref().map(String::as_str)
    }

    pub fn prev(&mut self) -> Option<&str> {
        if self.sessions.len() <= 1 {
            return None;
        }

        self.front_to_back();
        self.update_curr();
        self.sessions.front().map(String::as_str)
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.sessions.len() <= 1 {
            return None;
        }

        // SAFETY: we have at least 2 sessions
        let back = self.sessions.pop_back().unwrap();
        self.sessions.push_front(back);

        self.update_curr();
        self.sessions.front().map(String::as_str)
    }

    #[tracing::instrument]
    pub fn push<S>(&mut self, session: S)
        where S: AsRef<str> + Debug
    {
        let session = session.as_ref().to_string();
        self.cycle_to_session(&session);

        match self.sessions.front() {
            Some(front) if *front == session => { },
            _ => self.sessions.push_front(session)
        }

        self.update_curr();
    }
}
