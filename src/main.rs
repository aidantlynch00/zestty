use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;

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
    session_list: Option<Vec<SessionInfo>>,
    client_id: Option<u16>,
}

register_plugin!(Zestty);

#[derive(Debug, Serialize, Deserialize)]
struct SwitchArgs {
    name: Option<String>,
    path: Option<String>,
    layout: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
#[serde(rename_all = "snake_case")]
enum Command {
    Switch(SwitchArgs),
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
            EventType::ListClients,
        ];

        subscribe(events);
        tracing::info!("subscribed to {:?}", events);

        let permissions = &[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ];

        request_permission(permissions);
        tracing::info!("requested permissions {:?}", permissions);

        list_clients();
        tracing::info!("requested client list");
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
    #[tracing::instrument(skip_all)]
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::SessionUpdate(sessions, _) => {
                self.session_list = Some(sessions);
            },
            Event::ListClients(clients) => self.find_client(&clients),
            _ => { }
        }
    }

    #[tracing::instrument(skip_all)]
    fn handle_command(&mut self) {
        // do not handle the command before having info
        match (&self.session_list, &self.client_id) {
            (Some(_), Some(_)) => { },
            _ => {
                tracing::debug!("cannot handle command yet");
                return;
            }
        }

        if let Some(command) = self.buffered_command.take() {
            match command {
                Command::Switch(args) => self.switch(args),
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn switch(&self, args: SwitchArgs) {
        tracing::trace!("switch called");

        tracing::debug!("switching session with args {:?}", args);
        let SwitchArgs { name, path, layout } = args;

        let name = name.as_deref();
        let cwd = path.map(PathBuf::from);
        let layout = match layout {
            Some(layout) => LayoutInfo::File(layout),
            None => LayoutInfo::File(String::from("default"))
        };

        switch_session_with_layout(name, layout, cwd);
    }

    #[tracing::instrument(skip_all)]
    fn finish_setup(&mut self) {
        tracing::debug!("hiding plugin pane and making it unselectable");
        hide_self();
        set_selectable(false);

        while self.buffered_events.len() > 0 {
            let event = self.buffered_events.pop().unwrap();
            self.handle_event(event);
        }

        self.handle_command();
    }

    fn find_client(&mut self, clients: &Vec<ClientInfo>) {
        for client in clients {
            if client.is_current_client {
                self.client_id = Some(client.client_id);
            }
        }
    }
}
