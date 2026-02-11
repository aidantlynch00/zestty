# zestty
A POSIX-compliant shell script and accompanying [zellij](https://zellij.dev/) plugin for quickly moving between your project sessions.

## Features
- **Create** and **attach** to sessions from within a zellij session
- **List** sessionizable items
- **Sessionize** an item to create or attach to a session
- **Pick** an item from a list to sessionize
- **Cycle** through active sessions
- Jump **back** to the last session

TODO: add a GIF demonstrating core features

## Usage
> [!NOTE]
> Use `zestty help` to see the full help text. Run each command with no arguments to print its help text.

### Create and Attach
- `zestty create <name> [path] [layout]`: create a new session (works within a session)
- `zestty attach <name>`: attach to an existing session (works within a session)

### Session Lists
`zestty list` prints lists of sessionizable items.

#### zellij
- `zestty list active`: list active sessions
- `zestty list dead`: list dead sessions
- `zestty list zellij`: list all zellij sessions

#### Projects
- `zestty list projects`: list your projects
    - See instructions [below](#projects) to configure your project list

#### Git
- `zestty list worktrees`: list git worktrees
- `zestty list submodules`: list git submodules

> [!NOTE]
> Current working directory must be within a git repository

### Sessionize
Each sessionizable item has a corresponding sessionizer that creates or attaches to a zellij session.

- `zestty sessionize <item>`: runs the sessionizer associated with item, creating or attaching to a session

#### Smart Sessionizing
zestty is project-centric and employs a few tricks to make sessionizing feel smart.

- `session` and `project`: If the session is dead and its name matches that of a project, the session is deleted and recreated so that command panes execute immediately.
- `worktree`: If the path matches the path of a project, that project is sessionized.
> [!TIP]
> Use this to have a code review worktree with a layout designed for reviewing diffs.
- `submodule`:
    - If the name matches a project name, the project's layout is used when sessionizing the submodule.
    - If the path matches the path of a project, that project is sessionized.

### Picking Sessions
> [!NOTE]
> Requires [fzf](https://github.com/junegunn/fzf) to be available in your PATH

- `zestty pick`: fuzzy find over lists, sessionizing your selection

TODO: add GIF fuzzy finding over project list

### Session Cycling
zestty tracks the sessions you join in the order that you join them, allowing you to cycle through your active sessions. Attaching to a session already in the cycle moves you to that point in the cycle.

- `zestty previous`: switch to the previous session in the cycle
- `zestty next`: switch to the next session in the cycle

### Jump To Previous
zestty also tracks the last session you were attached to, allowing you to quickly jump back to your last session if it is active.

- `zestty back`: jump back to the last active session

## Configuration

### Projects
zestty does not scan your filesystem to find projects. Instead, you are expected to maintain a list of your projects, their locations, and optionally a zellij layout to apply when creating the session. Each line should be in the following format `name:path:layout` (example: `zestty:~/projects/zestty:edit-and-git`).

zestty looks in the following locations for a projects file:
1. ~/.config/zestty/projects
2. /etc/zestty/projects

### Config File
zestty looks in the following locations for a configuration file:
1. ~/.config/zestty/config
2. /etc/zestty/config

This file is sourced at runtime to configure zestty. All of the following values are also configurable via environment variable, which take precendence over the configuration file.

#### Delimiters
zestty allows the user to configure the delimiters used in session lines.

`ZESTTY_DELIM`: single character, changes the delimiter zestty uses when listing and sessionizing sessions (default ':').
`ZESTTY_PROJECT_DELIM`: single character, changes the delimiter zestty uses to split lines in the projects file (default ':').

#### Plugin URL
- `ZESTTY_PLUGIN_URL`: changes the plugin location zestty uses when communicating with the zestty plugin.
    - follows the zellij convention for plugin URLs
    - defaults to 'https://github.com/aidantlynch00/zestty/releases/latest/download/zestty.wasm'

> [!TIP]
> Download the zestty plugin and change `ZESTTY_PLUGIN_URL` to point to your local copy!

## Installation
A minimal installation of zestty only requires you to download the [zestty script](https://github.com/aidantlynch00/zestty/releases/latest/download/zestty). Since zellij can download plugins over HTTP and cache them, by default zestty uses the latest release of the plugin hosted on GitHub. A prebuilt plugin binary or an archive of both the script and plugin binary can be downloaded on the [releases page](https://github.com/aidantlynch00/zestty/releases/latest).

### From Source
To clone and build the plugin from source, run the following:

```sh
git clone https://github.com/aidantlynch00/zestty.git
cd zestty

# ensure you have the wasm32-wasip1 toolchain
rustup target add wasm32-wasip1
cargo build --release
```

The plugin binary will be available at target/wasm32-wasip1/release/zestty.wasm. Copy the zestty script to a location in your PATH and configure your [plugin URL](#plugin-url) to point to the newly built binary.

## Extending zestty
You can define custom shell functions to extend the listing and sessionizing capabilities of zestty. These custom shell functions run within the zestty environment and can make use of internal zestty functions. See [this page](INTERNALS.md) for the list of these functions and what they do.

> [!TIP]
> You can place these custom functions within your configuration file to make them available to zestty.

### Custom Session Lists
Define a `zestty_list_custom` shell function to make a custom list available via `zestty list custom`. Your list can consist of builtin item types or you can define your own custom item type.

For example, here's a custom function to list all directories under the current working directory with a maximum depth of 5 and a custom item type `dir`:
```sh
zestty_list_dirs() {
    find . -mindepth 1 -maxdepth 5 -type d |\
    sed "s/^/dir$ZESTTY_DELIM/"
}
```

### Custom Sessionizers
Define a `zestty_sessionize_custom` shell function to make a custom sessionizer for a `custom` item type. zestty splits the item by the configured delimiter and passes the result as the arguments to the sessionizer.

For example, here's a sessionizer for the `dir` item type we defined above:
```sh
zestty_sessionize_dir() {
    path=$1

    # Sessionize as project if the path matches
    project=$(zestty_match_project_path "$path")
    if [ -n "$project" ]; then
        zestty_sessionize "$project"
        return
    fi

    base=$(basename "$path")
    name="DIR_$base"
    state=$(zestty_get_session_state "$name")
    case "$state" in
        "dne") zestty_create "$name" "$path";;
        "dead" | "active") zestty_attach "$name";;
    esac
}
```

## Plugin
The zestty script pipes messages to the zestty plugin to enable certain features. While I recommend you use the zestty package as a whole, it is possible to use the zestty plugin standalone. See [this page](PLUGIN.md) for more information on how to use the zestty plugin.

## AI Use
AI use was kept to a minimum on this project. zestty was written "the old-fashioned way", with a few minor edits attributable to AI. Maybe this is obvious given that this project is not 50k lines long. Anything that makes its way to the main branch will have been reviewed by myself, always.

Reviewing the zestty script for POSIX compliance is where I found AI to be the most useful. Using [opencode](https://github.com/sst/opencode) with web tools and the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/), I was able have an agent comb through the zestty script, checking against the standard for compliance. When the agent found a violation, it had the standard in context and could offer suggestions for making my logic compliant. Additionally, opencode can send program output to the agent, allowing the agent to offer suggestions for fixing issues found by [shellcheck](https://github.com/koalaman/shellcheck).
