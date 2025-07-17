# Rust Text Editor

A lightweight, vim-like text editor built with Rust, featuring modal editing and terminal-based interface.

## Features

- Modal editing (Normal, Insert, Visual modes)
- Vim-inspired keybindings
- Terminal-based interface
- Fast and lightweight

## Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd rust_text_editor
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the editor:
   ```bash
   cargo run ./test.rs
   ```

## Usage

**Note: Currently, a file path must be provided as an argument for the editor to work properly.**

The editor supports standard vim-like navigation and editing commands in different modes:

- **Normal Mode**: Navigate and execute commands
- **Insert Mode**: Edit text content
- **Visual Mode**: Select and manipulate text

## Requirements

- Rust 1.70+ (or latest stable)
- Terminal with ANSI color support
