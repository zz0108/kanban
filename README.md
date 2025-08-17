# Kanban TUI

A terminal-based Kanban board application built with Rust and ratatui.

## Features

- **Multi-column Kanban board**: To Do, In Progress, Done columns
- **Task management**: Create, edit, and delete tasks
- **Task properties**: Title, description, priority levels
- **Navigation**: Vim-like keyboard controls (hjkl) or arrow keys
- **Data persistence**: Automatically saves/loads board state to JSON
- **Priority system**: Low, Medium, High, Critical with color indicators

## Installation

Make sure you have Rust installed, then:

### Method 1: Install globally
```bash
git clone <your-repo>
cd kanban
cargo install --path .
```

After installation, run anywhere with:
```bash
kanban
```

### Method 2: Build and run locally
```bash
git clone <your-repo>
cd kanban
cargo build --release
./target/release/kanban  # On Unix/Linux/macOS
# Or on Windows:
.\target\release\kanban.exe
```

## Development Commands

- `cargo run` - Run the Kanban TUI application
- `cargo build` - Build the project  
- `cargo check` - Check for compilation errors without building
- `cargo build --release` - Build optimized release version
- `cargo test` - Run unit tests (currently minimal test coverage)

## Usage

After installation, run the application:

```bash
kanban
```

Or during development:

```bash
cargo run
```

### Keyboard Controls

#### Normal Mode
- `h`/`j`/`k`/`l` or Arrow keys - Navigate between columns and tasks
- `n` - Create new task
- `Enter` - Edit selected task
- `d` - Delete selected task
- `m` - Enter move mode, `M` - Move to previous column
- `q` - Quit application

#### Edit/Add Mode
- Type: Enter text for current field
- `Tab`, `↓`: Next field
- `↑`: Previous field
- `+`/`=`: Increase priority, `-`: Decrease priority
- `Enter`: Confirm, `Esc`: Cancel

#### Move Mode
- `←`/`→`: Select target column
- `m`: Confirm move
- `Esc`: Cancel

### Priority Levels

Tasks are color-coded by priority:
- 🔴 Critical
- 🟡 High
- 🔵 Medium (default)
- 🟢 Low

## Architecture Overview

This application uses a **state-driven UI pattern** where:
- `App` struct holds all application state (board data, UI state, input modes)
- UI is purely a rendering function of the current state
- Input handlers mutate the application state
- Main loop coordinates rendering and input handling

### Key Components

- **App State Management** (`src/app.rs`): Central `App` struct manages all application state with `InputMode` enum handling different interaction modes
- **Data Models** (`src/models/`): Board, Column, and Task entities with UUID-based identification and serialization support
- **UI Rendering** (`src/ui.rs`): Pure function that renders current app state with modal popup system
- **Input Handling** (`src/handlers/input.rs`): Mode-based input routing with direct state mutations

### Data Flow

1. Main loop renders current app state
2. Input events are routed to appropriate handler based on current `InputMode`
3. Handlers directly mutate app state (selection indices, board data, input mode)
4. Changes trigger re-render on next loop iteration
5. Board state automatically persists to `kanban_board.db` on quit

## Data Storage

Board state is persisted to a SQLite database (`kanban_board.db`) with normalized tables (boards, columns, tasks). The storage is handled automatically and the database is created with proper foreign key relationships.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main application logic and state
├── ui.rs                # User interface rendering
├── models/
│   ├── mod.rs
│   ├── board.rs         # Kanban board data structure
│   ├── column.rs        # Column data structure
│   └── task.rs          # Task data structure
└── handlers/
    ├── mod.rs
    ├── input.rs         # Keyboard input handling
    └── storage.rs       # Data persistence
```

## Dependencies

- `ratatui` - TUI framework with `crossterm` backend for cross-platform terminal handling
- `rusqlite` - SQLite database with bundled SQLite for cross-platform compatibility
- `serde` - Serialization framework for data persistence
- `chrono` - Date/time handling
- `uuid` - Unique identifiers for tasks