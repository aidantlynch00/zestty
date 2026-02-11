# zestty Plugin
Communication with the zestty plugin is done via JSON-formatted payloads in piped messages. The plugin requires the `ReadApplicationState` and `ChangeApplicationState` permissions to operate. If these permissions are not granted, the zestty plugin closes itself.

## Usage
Send messages to the plugin using the `zellij pipe` command:

```sh
zellij pipe --plugin "https://github.com/aidantlynch00/zestty/releases/latest/download/zestty.wasm" -- <payload>
```

## Available Commands

### Switch Session
Creates or switches to a session by name with an optional working directory and zellij layout.

```json
{ "command": "switch-session", "name": "my-session", "path": "/path/to/dir", "layout": "default" }
```

**Parameters:**
- `name` (required): Session name to switch to or create
- `path` (optional): Working directory for the session
- `layout` (optional): Layout file to use

### Add Session to Cycle
Records the current session in the navigation history.

```json
{ "command": "add-session-to-cycle" }
```

> [!IMPORTANT]
> The `switch-session` command handles adding the requested session to the session cycle. This is what zestty uses when _inside_ of a zellij session. The `add-session-to-cycle` command is used by zestty when creating or attaching from _outside_ of a zellij session to keep the session cycle up-to-date.

### Cycle Previous
Switch to the previous session in the cycle history.

```json
{ "command": "cycle-previous" }
```

### Cycle Next
Switch to the next session in the cycle history.

```json
{ "command": "cycle-next" }
```

### Jump Back
Switch back to the last attached session.

```json
{ "command": "cycle-back" }
```

## Cycle Data
The plugin maintains the session cycle in `/tmp/zestty_sessions.json`. When switching sessions:

1. Dead sessions are automatically removed from the cycle
2. The target session is added to the cycle
3. The last attached session is updated
