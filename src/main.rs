mod history;

use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;
use history::{SessionHistory, LoadError, SaveError};

#[cfg(feature = "tracing")]
pub fn init_tracing() {
    use std::fs::File;
    use std::sync::Arc;
    use tracing_subscriber::layer::SubscriberExt;

    let file = File::create("/host/zestty.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("error creating log file: {:?}", error)
    };

    let writer = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(file));

    let subscriber = tracing_subscriber::registry()
        .with(writer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to init tracing");
}

#[derive(Default)]
struct Zestty {
    buffered_events: Vec<Event>,
    buffered_command: Option<Command>,
    permission_granted: Option<bool>,
    sessions: Option<Vec<SessionInfo>>,
    history: SessionHistory,
}

register_plugin!(Zestty);

#[derive(Debug, Serialize, Deserialize)]
struct SwitchSessionArgs {
    name: Option<String>,
    path: Option<String>,
    layout: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
#[serde(rename_all = "kebab-case")]
enum Command {
    AddSessionToHistory,
    SwitchSession(SwitchSessionArgs),
    PreviousSession,
    NextSession,
}

impl ZellijPlugin for Zestty {
    #[tracing::instrument(skip_all)]
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        #[cfg(feature = "tracing")]
        {
            init_tracing();
            tracing::debug!("tracing initialized");
        }

        let events = &[
            EventType::PermissionRequestResult,
            EventType::SessionUpdate,
        ];

        subscribe(events);
        tracing::info!("subscribed to {:?}", events);

        let permissions = &[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ];

        request_permission(permissions);
        tracing::info!("requested permissions {:?}", permissions);
    }

    #[tracing::instrument(skip_all)]
    fn update(&mut self, event: Event) -> bool {
        match (&self.permission_granted, &event) {
            (None, Event::PermissionRequestResult(PermissionStatus::Granted)) => {
                tracing::info!("permission granted");
                self.permission_granted = Some(true);
                self.finish_setup();
            },
            (_, Event::PermissionRequestResult(PermissionStatus::Denied)) => {
                tracing::info!("permission denied, closing");
                self.permission_granted = Some(false);
                close_self();
            },
            (None, _) => {
                self.buffered_events.push(event);
            },
            (Some(true), _) => {
                self.handle_event(event);
            },
            (Some(false), _) => { }
        }

        false
    }

    #[tracing::instrument(skip_all)]
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let payload = match pipe_message.payload {
            Some(payload) => payload,
            None => return false
        };

        let command = match serde_json::from_str::<Command>(&payload) {
            Ok(command) => command,
            Err(de_err) => {
                tracing::error!("could not deserialize command: {}", de_err);
                return false;
            },
        };

        self.buffered_command = Some(command);
        self.handle_command();

        false
    }

    #[tracing::instrument(skip_all)]
    fn render(&mut self, _rows: usize, _cols: usize) { }
}

impl Zestty {
    const HISTORY_PATH: &'static str = "/tmp/zestty_history.json";

    #[tracing::instrument(skip_all)]
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::SessionUpdate(sessions, _) => {
                tracing::debug!("handling session update event");
                self.sessions = Some(sessions);
            },
            _ => { }
        }

        self.handle_command();
    }

    #[tracing::instrument(skip_all)]
    fn handle_command(&mut self) {
        // do not handle the command before having session list
        if self.sessions.is_none() {
            tracing::debug!("cannot handle command yet");
            return;
        };

        if let Some(command) = self.buffered_command.take() {
            self.load_history();

            match command {
                Command::AddSessionToHistory => self.add_session_to_history(),
                Command::SwitchSession(args) => self.switch_session(args),
                Command::PreviousSession => self.prev_session(),
                Command::NextSession => self.next_session(),
            }

            self.save_history();
            close_self();
        }
    }

    #[tracing::instrument(skip_all)]
    fn add_session_to_history(&mut self) {
        tracing::trace!("add_session_to_history called");

        // SAFETY: we have the session list and one must be active
        let session_name = self.find_session().unwrap();

        // update history
        self.history.add_session(session_name);
    }

    #[tracing::instrument(skip_all)]
    fn switch_session(&mut self, args: SwitchSessionArgs) {
        tracing::trace!("switch_session called");

        tracing::debug!("switching session with args {:?}", args);
        let SwitchSessionArgs { name, path, layout } = args;

        let name = name.as_deref();
        let cwd = path.map(PathBuf::from);
        let layout = match layout {
            Some(layout) => LayoutInfo::File(layout),
            None => LayoutInfo::File(String::from("default"))
        };

        // TODO: if session already exists, we will not load plugin to add to stack,
        // so add here

        switch_session_with_layout(name, layout, cwd);
    }

    #[tracing::instrument(skip_all)]
    fn prev_session(&mut self) {
        tracing::trace!("prev_session called");

        match self.history.prev() {
            session @ Some(_) => switch_session(session),
            None => tracing::debug!("no previous session")
        }
    }

    #[tracing::instrument(skip_all)]
    fn next_session(&mut self) {
        tracing::trace!("next_session called");

        match self.history.next() {
            session @ Some(_) => switch_session(session),
            None => tracing::debug!("no next session")
        }
    }

    #[tracing::instrument(skip_all)]
    fn finish_setup(&mut self) {
        tracing::debug!("hiding plugin pane and making it unselectable");
        hide_self();
        set_selectable(false);

        // plugin load was due to session startup, not from a piped command so
        // add current session to the history stack
        if self.buffered_command.is_none() {
            self.buffered_command = Some(Command::AddSessionToHistory);
        }

        while self.buffered_events.len() > 0 {
            let event = self.buffered_events.pop().unwrap();
            self.handle_event(event);
        }
    }

    #[tracing::instrument(skip_all)]
    fn load_history(&mut self) {
        let path = PathBuf::from(Zestty::HISTORY_PATH);
        match SessionHistory::load_from_file(path) {
            Ok(history) => {
                tracing::debug!("loaded history: {:?}", history);
                self.history = history;
            },
            Err(LoadError::FileNotFound) =>
                tracing::debug!("no existing history"),
            Err(LoadError::CannotOpenFile(io_err)) =>
                tracing::error!("could not open history file: {}", io_err),
            Err(LoadError::CannotDeserialize(de_err)) =>
                tracing::error!("could not deserialize history: {}", de_err),
        }
    }

    #[tracing::instrument(skip_all)]
    fn save_history(&self) {
        let path = PathBuf::from(Zestty::HISTORY_PATH);
        match self.history.save_to_file(path) {
            Ok(()) => { },
            Err(SaveError::CannotCreateFile(io_err)) =>
                tracing::error!("could not create history file: {}", io_err),
            Err(SaveError::CannotSerialize(se_err)) =>
                tracing::error!("could not serialize history: {}", se_err),
        }
    }

    fn find_session(&mut self) -> Option<String> {
        for session in self.sessions.as_ref()? {
            if session.is_current_session {
                return Some(session.name.clone())
            }
        }

        None
    }
}
