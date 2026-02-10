# zestty Internals

## Variables

| Variable | Description |
|----------|-------------|
| `ZESTTY_DELIM` | Delimiter for internal item format. |
| `ZESTTY_PROJECT_DELIM` | Delimiter for project file entries. |
| `ZESTTY_PLUGIN_URL` | Configured zestty plugin URL. |
| `ZESTTY_CWD` | Current working directory captured at script startup |

## Functions

| Function | Inputs | Outputs | Description |
|----------|--------|---------|-------------|
| `zestty_find_file_in_order` | File paths | First existing readable file path | Searches for files in order and returns the first readable one. |
| `zestty_find_config_file` | N/A | Config file path or empty | Finds zestty configuration file if it exists. |
| `zestty_find_project_file` | N/A | Projects file path or empty | Finds zestty projects file if it exists. |
| `zestty_function_exists` | `1`: function name | Exit code 0 if it exists, 1 otherwise | Checks if a function with the given name is defined in the current shell environment. |
| `zestty_get_session_state` | `1`: session name | One of `active`, `dead`, or `dne` (does not exist) | Checks if a zellij session exists and returns its state. |
| `zestty_call_switch_session` | `1`: name<br>`2`: path (optional)<br>`3`: layout (optional) | N/A | Pipes message to plugin to switch/create session inside an existing zellij session. |
| `zestty_match_project_name` | `1`: project name | Project item or empty | Looks up project by name in projects file. |
| `zestty_match_project_path` | `1`: project path | Project item or empty | Looks up project by path in projects file. |
| `zestty_create` | `1`: name<br>`2`: path (optional)<br>`3`: layout (optional) | N/A | Creates a new zellij session with specified parameters. |
| `zestty_attach` | `1`: session name | N/A | Attaches to an existing zellij session. |
| `zestty_list_active` | N/A | Active session items | Lists all active zellij sessions. |
| `zestty_list_dead` | N/A | Dead session items | Lists all dead zellij sessions. |
| `zestty_list_projects` | N/A | Project items | Lists all projects from the projects file. |
| `zestty_list_worktrees` | N/A | Worktree items | Lists all git worktrees in the current repository. |
| `zestty_list_submodules` | N/A | Submodule items | Lists all git submodules in the current repository. |
| `zestty_list` | List types | List items | Lists items based on specified types. |
| `zestty_split_string` | `1`: string | No output, sets shell arguments | Splits a delimited string into shell-quoted arguments. |
| `zestty_sessionize_session` | `1`: session name | N/A | Sessionizer for `session` type items. |
| `zestty_sessionize_project` | `1`: name<br>`2`: path<br>`3`: layout (optional) | N/A | Sessionizer for `project` type items. |
| `zestty_sessionize_worktree` | `1`: ref<br>`2`: path | N/A | Sessionizer for `worktree` type items. |
| `zestty_sessionize_submodule` | `1`: name<br>`2`: path | N/A | Sessionizer for `submodule` type items. |
| `zestty_sessionize` | `1`: item string | N/A | Splits the item and calls the appropriate sessionizer. |
| `zestty_previous` | N/A | N/A | Cycles to the previous session via piped message to plugin. |
| `zestty_next` | N/A | N/A | Cycles to the next session via piped message to plugin. |
| `zestty_back` | N/A | N/A | Jumps back to the last session via piped message to plugin. |
