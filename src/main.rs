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
    permission_granted: bool,
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
        match event {
            Event::PermissionRequestResult(status) => {
                self.permission_granted = matches!(status, PermissionStatus::Granted);
            },
            _ => {}
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

        match command {
            Command::Switch(args) => self.switch(args),
        }

        false
    }

    #[tracing::instrument(skip_all)]
    fn render(&mut self, _rows: usize, _cols: usize) { }
}

impl Zestty {
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

        // close the plugin window if the user has already granted permissions
        // to zestty, otherwise, keep it open for that purpose
        if self.permission_granted {
            close_self();
        }
    }
}
