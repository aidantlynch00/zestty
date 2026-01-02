# zestty
A POSIX-compliant shell script and accompanying [zellij](https://zellij.dev/) plugin for moving between your project sessions.

## Usage
Use `zestty help` to see the full help text.

### Creating and Attaching To Sessions
zestty offers `zestty create` and `zestty attach` commands as replacements for the zellij equivalents, as they work within a zellij session.

TODO: add GIF creating a session within a session

### Session Lists

TODO: add GIF listing project sessions

### Sessionize

## Fuzzy Finding

TODO: add GIF fuzzy finding in project list

## Configuration

### Projects
zestty does not scan your filesystem to find projects. Instead, you are expected to maintain a list of your projects, their locations, and optionally a zellij layout to apply when creating the session. Each line should be in the following format `name:path:layout` (example: `zestty:~/projects/zestty:edit-and-git`). zestty looks in the following locations for a projects file:
1. ~/.config/zestty/projects
2. /etc/zestty/projects

### Config File
zestty looks in the following locations for a configuration file:
1. ~/.config/zestty/config
2. /etc/zestty/config

This file is sourced at runtime to configure certain values. All of the following values are also available for configuration via environment variable.

#### Delimiters
zestty allows the user to configure the delimiters used in session lines.

`ZESTTY_DELIMITER`: changes the delimiter zestty uses when listing and sessionizing sessions (default ':').
`ZESTTY_PROJECT_DELIMITER`: changes the delimiter zestty uses to split lines in the projects file (default ':').

#### Plugin URL
`ZESTTY_PLUGIN_URL`: changes the plugin location zestty uses when communicating with the zestty plugin (default 'https://github.com/aidantlynch00/zestty/releases/latest/download/zestty.wasm').

The zestty script defaults to using the latest GitHub release of the zestty plugin when piping commands to it, removing the need to add the plugin to your zellij configuration. However, I prefer to have my software available offline just in case, so I use `ZESTTY_PLUGIN_URL` to point to an offline copy of the plugin. I wrote the source, I better be using the binary I have built on my machine!

## Extending zestty
Coming soon!
- Custom Session Lists
- Custom Sessionizers

## AI Use
AI use was kept to a minimum on this project. zestty was written "the old-fashioned way", with a few minor edits attributable to AI. Anything that makes its way to the main branch will have been reviewed by myself.

Reviewing the zestty script for POSIX compliance is where I found AI to be the most useful. Using [opencode](https://github.com/sst/opencode) with web tools and the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/), I was able have an agent comb through the zestty script, checking against the standard for compliance. When the agent found a violation, it had the standard in context and could offer suggestions for making my logic compliant. Additionally, opencode can send program output to the agent, allowing the agent to offer suggestions for fixing issues found by [shellcheck](https://github.com/koalaman/shellcheck).
