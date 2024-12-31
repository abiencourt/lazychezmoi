# Lazychezmoi

**Lazychezmoi** is a [Ratatui](https://ratatui.rs) app to manage your [chezmoi](https://chezmoi.io) dotfiles.

Heavily inspired by [lazygit](https://github.com/jesseduffield/lazygit).

## Table of contents

<!-- toc -->

- [Features](#features)
  - [Coloured diff and file states](#coloured-diff-and-file-states)
  - [Interactive file management](#interactive-file-management)
  - [Integrated chezmoi commands](#integrated-chezmoi-commands)
- [Usage](#usage)
  - [Keybindings](#keybindings)

<!-- tocstop -->

## Features

### Coloured diff and file states

- `chezmoi status` with colour-coded file states
- Coloured diff view

### Interactive file management

- Select/deselect files using <space>
- Add/Re-add selected files to chezmoi source directory
- View detailed diff for each single file

### Integrated chezmoi commands

- Shortcut to open chezmoi source directory (i.e. `chezmoi edit`)
- Shortcut to edit a file in the chezmoi source (i.e. `chezmoi edit <file>`)

## Usage

### Keybindings

- `↑/k`: Navigate up
- `↓/j`: Navigate down
- `Space`: Toggle file selection
- `e`: Edit highlighted file
- `a`: Add/re-add selected files
- `A`: Apply selected files
- `S`: Open chezmoi source directory
- `q/Esc`: Quit application
