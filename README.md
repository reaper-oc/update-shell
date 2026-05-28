# updSH

A Linux shell with a built-in package manager, written in Rust.

## Features

- Readline-based REPL with tab completion and fish-style autosuggestions
- Command history with file persistence
- Job control (background jobs, fg, bg)
- Pipes, redirections, variables, and command sequencing
- Alias system with file persistence
- Customizable prompt (multiline, powerline, minimal, plain)
- Config file at `~/.config/updsh/env`
- Package manager (install, remove, search, update)

## Installation

```
git clone git@github.com:reaper-oc/update-shell.git
cd update-shell
./install.sh
```

The install script builds the binary and places it at `~/.local/bin/updsh`.

## Requirements

- Rust toolchain (rustc, cargo)
- Linux environment (uses libc syscalls)
- Libraries: libc, pthreads (system defaults)

## Package Manager

updSH includes a package manager (`pkg`) that distributes three types of
packages from a built-in registry and a remote Vercel-hosted registry:

- **source**: Shell alias scripts sourced on shell startup
- **compile**: C or Rust source compiled with gcc/rustc
- **build**: Git repositories built with cmake/cargo/make

```
pkg install <name>
pkg remove <name>
pkg list
pkg search [query]
pkg info <name>
pkg update
```

Packages are scanned for malicious patterns and automatically rejected.

## Configuration

Config file at `~/.config/updsh/env`:

```
UPD_PROMPT_STYLE=multiline
UPD_COLOR_USER=green
UPD_COLOR_HOST=blue
UPD_COLOR_PATH=yellow
UPD_COLOR_GIT=red
UPD_COLOR_EXIT=red
UPD_SHOW_GIT=yes
UPD_SHOW_EXIT_CODE=yes
```

## Prompt Styles

- **multiline**: Two-line prompt with a decorative border
- **powerline**: Colored background segments
- **minimal**: Compact single line with git branch
- **plain**: Simple path and prompt character

## Built-in Commands

| Command | Description |
|---------|-------------|
| `cd <dir>` | Change directory |
| `exit [code]` | Exit the shell |
| `pwd` | Print working directory |
| `echo [text]` | Print arguments |
| `clear` | Clear terminal |
| `export KEY=VALUE` | Set environment variable |
| `history` | Show command history |
| `help` | Open documentation in browser |
| `jobs` | List background jobs |
| `fg [id]` | Bring job to foreground |
| `bg [id]` | Resume job in background |
| `source <file>` | Execute commands from file |
| `type <cmd>` | Show command type |
| `alias [name=value]` | Define or list aliases |
| `unalias <name>` | Remove an alias |
| `pkg <command>` | Package manager |

## Shell Features

- Tab completion for commands, paths, and package names
- Autosuggestions based on command history
- Pipes: `cmd1 | cmd2`
- Redirections: `>`, `>>`, `<`, `2>`, `2>&1`
- Background: `cmd &`
- Sequencing: `cmd1; cmd2`
- Variables: `$VAR`, `${VAR}`
- Signal handling: SIGINT, SIGCHLD, SIGTSTP, SIGTTIN, SIGTTOU

## Project Structure

```
src/
  main.rs        Entry point and REPL loop
  parser.rs      Command line tokenizer
  executor.rs    Fork/exec, pipes, redirects, job control
  builtins.rs    Built-in command implementations
  pkg.rs         Package manager (install, build, compile)
  alias.rs       Alias store and expansion
  prompt.rs      Prompt rendering (four styles)
  config.rs      Config file loader
  completer.rs   Tab completion and autosuggestions
  history.rs     Command history with file persistence
  job.rs         Job control structures
  signal.rs      Signal handler setup
  help.html      Embedded documentation
  builtin_packages.json  82 built-in packages
install.sh       Build and install script
```

## License

MIT
