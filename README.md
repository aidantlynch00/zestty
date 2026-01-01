# zestty
A POSIX-compliant shell script for moving between [zellij](https://zellij.dev/) sessions for your projects.

## Dependencies
zestty currently relies on the [zellij-switch](https://github.com/mostafaqanbaryan/zellij-switch) plugin to change sessions while in a zellij session. The simplest way to set it up is to add the following to the plugins sections of your zellij config:
```kdl
plugins {
    zellij-switch location="https://github.com/mostafaqanbaryan/zellij-switch/releases/latest/download/zellij-switch.wasm"
}
```

## Usage
Use `zestty help` to see the full help text.

### Creating and Attaching To Sessions
zestty offers `zestty create` and `zestty attach` commands as replacements for the zellij equivalents, as they work within a zellij session.

TODO: add GIF creating a session within a session

### Session Lists

### Sessionize

## Fuzzy Finding

## Configuration

### Projects
zestty does not scan the user's filesystem to find their projects. Instead, the user is expected to maintain a list of their projects, the project's locations, and optionally a zellij layout to apply when creating the session. Each line should be in the following format `name:path:layout` (example: `zestty:~/projects/zestty:edit-and-git`). zestty looks in the following locations for a projects file:
1. ~/.config/zestty/projects
2. /etc/zestty/projects

### Delimiters
zestty also allows the user to configure the delimiters used in session lines.

`ZESTTY_DELIMITER`: changes the delimiter zestty uses when listing and sessionizing sessions (default ':').
`ZESTTY_PROJECT_DELIMITER`: changes the delimiter zestty uses to split lines in the projects file (default ':').

These can either be configured as environment variables or within a file sourced by zestty at runtime. zestty looks in the following locations for a configuration file:
1. ~/.config/zestty/config
2. /etc/zestty/config

## Extending zestty

### Custom Session Lists

### Custom Sessionizers

## AI Use
AI use was kept to a minimum on this project. zestty was written "the old-fashioned way", with a few minor edits attributable to AI. Anything that makes its way to the main branch will have been reviewed by myself.

Reviewing zestty for POSIX compliance is where I found AI to be the most useful. Using [opencode](https://github.com/sst/opencode) with web tools and the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/), I was able have an agent comb through zestty, checking against the standard for compliance. When the agent found a violation, it had the standard in context and could offer suggestions for making my logic compliant. Additionally, opencode can send program output to the agent, allowing the agent to offer suggestions for fixing issues found by [shellcheck](https://github.com/koalaman/shellcheck).
