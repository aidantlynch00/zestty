# zestty
A POSIX-compliant shell script and accompanying [zellij](https://zellij.dev/) plugin for moving between your project sessions.

## Usage
Use `zestty help` to see the full help text. Run each command with no arguments to print its help text.

### Creating and Attaching To Sessions
zestty offers `zestty create` and `zestty attach` commands as replacements for the zellij equivalents that work within a zellij session.

TODO: add GIF creating a session within a session

### Session Lists
zestty prints several different lists of sessionizable objects. There are several different things that can be sessionized:
- zellij sessions
    - active sessions
    - dead sessions
- projects
- git (must be in git repo)
    - worktrees
    - submodules

TODO: add GIF listing project sessions

### Sessionize
Each sessionizable object has a corresponding sessionizer that creates or attaches to a zellij session. Simply pass a line from one of the session lists to move to that session. zestty is project-centric and employs a few a tricks to make sessionizing feel smart.

For dead zellij sessions, if the name matches a project name, the project session is deleted and recreated. This is preferred to zellij session resurrection, which prevents commands from running immediately on attach. There is the `--force-run-commands` flag to `zellij attach`, but I find that is not a sensible default in case I unknowingly had a destructive command running in a pane. This happens when sessionizing both the `session` and `project` types.

For the `worktree` type, if the path matches the path of a project, it is sessionized as that project. I personally use this to have a code review worktree that has a different layout for reviewing changes.

For the `submodule` type, if the name matches a project name, the project's layout is used when sessionizing and the new session will be in the submodule directory. Like with worktrees, if the path matches the path of a project, it is sessionized as that project.

## Fuzzy Finding
`zestty list` and `zestty sessionize` can be married with a fuzzy finder to build pickers for your sessions. The following shell function can be used to pick over your projects:

```sh
# TODO: project picker
```

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

`ZESTTY_DELIM`: changes the delimiter zestty uses when listing and sessionizing sessions (default ':').
`ZESTTY_PROJECT_DELIM`: changes the delimiter zestty uses to split lines in the projects file (default ':').

#### Plugin URL
`ZESTTY_PLUGIN_URL`: changes the plugin location zestty uses when communicating with the zestty plugin (default 'https://github.com/aidantlynch00/zestty/releases/latest/download/zestty.wasm').

The zestty script defaults to using the latest GitHub release of the zestty plugin when piping commands to it, removing the need to add the plugin to your zellij configuration. However, I prefer to have my software available offline just in case, so I use `ZESTTY_PLUGIN_URL` to point to an offline copy of the plugin. I wrote the source, I better be using the binary I have built on my machine!

## Extending zestty
Coming soon!
- Custom Session Lists
- Custom Sessionizers

## Why zestty? (TODO)
- wanted to move between sessions
- wanted to attach to sessions in and outside of zellij
- wanted to define a list of projects with paths and layouts

## AI Use
AI use was kept to a minimum on this project. zestty was written "the old-fashioned way", with a few minor edits attributable to AI. Anything that makes its way to the main branch will have been reviewed by myself. Maybe this is obvious given that this all can fit in under 1,000 lines.

Reviewing the zestty script for POSIX compliance is where I found AI to be the most useful. Using [opencode](https://github.com/sst/opencode) with web tools and the [POSIX standard](https://pubs.opengroup.org/onlinepubs/9699919799/), I was able have an agent comb through the zestty script, checking against the standard for compliance. When the agent found a violation, it had the standard in context and could offer suggestions for making my logic compliant. Additionally, opencode can send program output to the agent, allowing the agent to offer suggestions for fixing issues found by [shellcheck](https://github.com/koalaman/shellcheck).
