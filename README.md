# Snake

`snake` is a small Rust CLI for saving shell commands under memorable aliases,
searching them interactively, and running the selected command.

## Install

Install directly from GitHub with Cargo:

```bash
cargo install --git https://github.com/ma-cohen/scripts-snake
```

## Update

Reinstall from GitHub with `--force`:

```bash
cargo install --git https://github.com/ma-cohen/scripts-snake --force
```

## Usage

Add a command:

```bash
snake add
```

You will be prompted for an alias and the command to save.

Search and run a saved command:

```bash
snake
```

Start the picker with an initial query:

```bash
snake run build
```

If `build` is an exact alias, `snake` runs it immediately. Otherwise, it opens
the searchable picker with `build` prefilled.

List aliases:

```bash
snake list
```

Remove an alias:

```bash
snake remove build
```

Or choose an alias to remove interactively:

```bash
snake remove
```

Press `Esc` to exit interactive selection without running or removing anything.

## Storage

Aliases are stored in a user config file managed by your operating system, so
reinstalling the binary does not remove your saved commands.
