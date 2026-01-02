use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Default)]
struct Zestty;

register_plugin!(Zestty);

#[derive(Serialize, Deserialize)]
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
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
    }

    fn update(&mut self, _event: Event) -> bool { false }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let payload = match pipe_message.payload {
            Some(payload) => payload,
            None => return false
        };

        let command = match serde_json::from_str::<Command>(&payload) {
            Ok(command) => command,
            Err(de_err) => {
                eprintln!("could not deserialize command: {}", de_err);
                return false;
            },
        };

        match command {
            Command::Switch(args) => self.switch(args),
        }

        return false;
    }

    fn render(&mut self, _rows: usize, _cols: usize) { }
}

impl Zestty {
    fn switch(&self, args: SwitchArgs) {
        let SwitchArgs { name, path, layout } = args;
        let name = name.as_deref();
        let cwd = path.map(PathBuf::from);

        if let Some(layout) = layout {
            let layout = LayoutInfo::File(layout);
            switch_session_with_layout(name, layout, cwd);
        }
        else {
            switch_session_with_cwd(name, cwd);
        }
    }
}
