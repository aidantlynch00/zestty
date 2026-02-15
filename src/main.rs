mod cycle;
mod version;

use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;
use cycle::{SessionCycle, LoadError, SaveError};
use version::{CompatibilityInfo, SemanticVersion};

#[cfg(feature = "tracing")]
pub fn init_tracing() {
    use std::fs::File;
    use std::sync::Arc;
    use tracing_subscriber::layer::SubscriberExt;

    let file = File::create("/tmp/zestty.log");
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
    compat_info: CompatibilityInfo,
    buffered_events: Vec<Event>,
    buffered_command: Option<Command>,
    permission_granted: Option<bool>,
    sessions: Option<Vec<SessionInfo>>,
    cycle: SessionCycle,
}

register_plugin!(Zestty);

#[derive(Debug, Serialize, Deserialize)]
struct SwitchSessionArgs {
    name: String,
    path: Option<String>,
    layout: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
#[serde(rename_all = "kebab-case")]
enum Command {
    AddSessionToCycle,
    SwitchSession(SwitchSessionArgs),
    CyclePrevious,
    CycleNext,
    CycleBack,
}

impl ZellijPlugin for Zestty {
    #[tracing::instrument(skip_all)]
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        #[cfg(feature = "tracing")]
        init_tracing();
        tracing::debug!("tracing initialized");

        self.set_compatibility();

        // show plugin pane on load
        show_self(true);

        // do not subscribe to events or request permissions if not compatible
        if !self.compat_info.compatible() {
            tracing::error!("versions are incompatible");
            return;
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
    fn render(&mut self, _rows: usize, _cols: usize) {
        // do not render if versions are compatible
        if self.compat_info.compatible() {
            tracing::info!("rendering compatibility info");
            return;
        }

        let min_version = format!("{}   ", self.compat_info.min_version);

        let (help_text, actual_version) = match &self.compat_info.actual_version {
            Some(version) =>
                ("Minimum zellij version not met!", format!("{}", version)),
            None =>
                ("Could not parse zellij version!", String::default()),
        };

        print_text(Text::new(help_text).color_all(1));

        let table = Table::new()
            .add_row(vec!["Minimum Version   ", "Actual Version"])
            .add_styled_row(vec![
                Text::new(min_version).color_all(0),
                Text::new(actual_version).color_all(0)
            ]);

        print_table_with_coordinates(table, 0, 2, None, None);
    }
}

impl Zestty {
    const SESSIONS_FILE: &'static str = "/tmp/zestty_sessions.json";

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
            self.load_sessions();

            // SAFETY: check for none value happens above
            let sessions = self.sessions.as_ref().unwrap();
            self.cycle.remove_dead_sessions(sessions);

            match command {
                Command::AddSessionToCycle => self.add_session_to_cycle(),
                Command::SwitchSession(args) => self.switch_session(args),
                Command::CyclePrevious => self.cycle_previous(),
                Command::CycleNext => self.cycle_next(),
                Command::CycleBack => self.cycle_back(),
            }

            self.save_sessions();
            close_self();
        }
    }

    #[tracing::instrument(skip_all)]
    fn add_session_to_cycle(&mut self) {
        // SAFETY: we have the session list and one must be active
        let session_name = self.find_session().unwrap();

        // update cycle
        self.cycle.push(session_name);
    }

    #[tracing::instrument(skip_all)]
    fn switch_session(&mut self, args: SwitchSessionArgs) {
        tracing::debug!("switching session with args {:?}", args);
        let SwitchSessionArgs { name, path, layout } = args;

        let cwd = path.map(PathBuf::from);
        let layout = match layout {
            Some(layout) => LayoutInfo::File(layout),
            None => LayoutInfo::File(String::from("default"))
        };

        // add the session to the cycle
        self.cycle.push(name.clone());

        switch_session_with_layout(Some(&name), layout, cwd);
    }

    #[tracing::instrument(skip_all)]
    fn cycle_previous(&mut self) {
        match self.cycle.prev() {
            session @ Some(name) => {
                tracing::debug!("switching to session '{}'", name);
                switch_session(session);
            },
            None => tracing::debug!("no previous session")
        }
    }

    #[tracing::instrument(skip_all)]
    fn cycle_next(&mut self) {
        match self.cycle.next() {
            session @ Some(name) => {
                tracing::debug!("switching to session '{}'", name);
                switch_session(session);
            },
            None => tracing::debug!("no next session")
        }
    }

    #[tracing::instrument(skip_all)]
    fn cycle_back(&mut self) {
        match self.cycle.back() {
            session @ Some(name) => {
                tracing::debug!("switching to session '{}'", name);
                switch_session(session);
            },
            None => tracing::debug!("no session to go back to")
        }
    }

    #[tracing::instrument(skip_all)]
    fn finish_setup(&mut self) {
        match self.buffered_command {
            Some(_) => {
                tracing::debug!("hiding plugin pane and making it unselectable");
                hide_self();
                set_selectable(false);
            },
            None => {
                tracing::debug!("no command, closing plugin pane");
                close_self();
            }
        }

        while self.buffered_events.len() > 0 {
            let event = self.buffered_events.pop().unwrap();
            self.handle_event(event);
        }
    }

    #[tracing::instrument(skip_all)]
    fn load_sessions(&mut self) {
        let path = PathBuf::from(Zestty::SESSIONS_FILE);
        match SessionCycle::load_from_file(path) {
            Ok(cycle) => {
                tracing::info!("loaded sessions: {:?}", cycle);
                self.cycle = cycle;
            },
            Err(LoadError::FileNotFound) =>
                tracing::info!("no existing sessions"),
            Err(LoadError::CouldNotOpenFile(io_err)) =>
                tracing::error!("could not open sessions file: {}", io_err),
            Err(LoadError::CouldNotDeserialize(de_err)) =>
                tracing::error!("could not deserialize sessions: {}", de_err),
        }
    }

    #[tracing::instrument(skip_all)]
    fn save_sessions(&self) {
        let path = PathBuf::from(Zestty::SESSIONS_FILE);
        match self.cycle.save_to_file(&path) {
            Ok(()) =>
                tracing::debug!("saved sessions to '{}'", path.display()),
            Err(SaveError::CouldNotCreateFile(io_err)) =>
                tracing::error!("could not create sessions file: {}", io_err),
            Err(SaveError::CouldNotSerialize(se_err)) =>
                tracing::error!("could not serialize sessions: {}", se_err),
        }
    }

    #[tracing::instrument(skip_all)]
    fn set_compatibility(&mut self) {
        let version = get_zellij_version();
        let version = version.as_str();
        tracing::debug!("zellij version: {}", version);

        let version = SemanticVersion::try_from(version).ok();
        self.compat_info = CompatibilityInfo::new(version);
    }

    fn find_session(&self) -> Option<String> {
        for session in self.sessions.as_ref()? {
            if session.is_current_session {
                return Some(session.name.clone())
            }
        }

        None
    }
}
