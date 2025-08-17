# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

- `cargo run` - Run the Kanban TUI application
- `cargo build` - Build the project  
- `cargo check` - Check for compilation errors without building
- `cargo build --release` - Build optimized release version
- `cargo test` - Run unit tests (currently minimal test coverage)
- `cargo test [TESTNAME]` - Run specific tests matching the name pattern
- `cargo fmt` - Format code using rustfmt
- `cargo fmt --check` - Check if code is formatted (useful for CI)
- `cargo clippy` - Run Clippy linter for code quality checks
- `cargo clippy --fix` - Automatically fix clippy suggestions
- `cargo install --path .` - Install the `kanban` command globally (must be run in project directory)
- `kanban` - Run the installed application (after installation)
- `./target/release/kanban` - Run the locally built binary (Unix/Linux/macOS)
- `.\target\release\kanban.exe` - Run the locally built binary (Windows)

## Architecture Overview

This is a terminal-based Kanban board application built with Rust and ratatui. The architecture follows a clear separation of concerns:

### Core Architecture Pattern

The application uses a **state-driven UI pattern** where:
- `App` struct holds all application state (board data, UI state, input modes)
- UI is purely a rendering function of the current state
- Input handlers mutate the application state
- Main loop coordinates rendering and input handling

### Key Components

**App State Management (`src/app.rs`)**:
- Central `App` struct manages all application state
- `InputMode` enum handles different interaction modes (Normal, Editing, AddingTask)
- Selection tracking via `selected_column` and `selected_task` indices
- Ephemeral `EditState` for form input during task creation/editing

**Data Models (`src/models/`)**:
- **Board**: Contains multiple columns, provides task movement operations between columns
- **Column**: Contains tasks, provides task manipulation within column  
- **Task**: Core entity with UUID, title, description, priority, and timestamps
- All models are `Serialize`/`Deserialize` for SQLite persistence

**UI Rendering (`src/ui.rs`)**:
- Pure function that renders current app state
- Modal popup system for task editing overlaid on main board view
- Color-coded priority indicators and selection highlighting

**Input Handling (`src/handlers/input.rs`)**:
- Mode-based input routing (Normal vs Editing modes have different key bindings)
- Direct state mutations on the App struct
- Navigation uses array indices to track selection
- In editing mode: Up/Down arrows and Tab navigate between form fields (Title, Description, Priority)

### Data Flow

1. Main loop renders current app state
2. Input events are routed to appropriate handler based on current `InputMode`
3. Handlers directly mutate app state (selection indices, board data, input mode)
4. Changes trigger re-render on next loop iteration
5. Board state automatically persists to `kanban_board.db` on quit

### Storage Format

Board state is persisted to a SQLite database (`kanban_board.db`) with normalized tables (boards, columns, tasks). The storage handler (`src/handlers/storage.rs`) provides save/load operations that the main loop calls automatically. The database is created automatically with proper foreign key relationships.

### Task Movement Logic

Tasks are moved between columns by:
1. Removing task from source column by UUID
2. Adding task to target column  
3. Updating selection indices if the current column lost tasks

This requires careful coordination between Board, Column operations and UI selection state management.

## Dependencies and External Integration

**Database Storage**: Uses SQLite via `rusqlite` crate with bundled SQLite for cross-platform compatibility. Database schema includes foreign key constraints and proper indexing.

**TUI Framework**: Built on `ratatui` with `crossterm` backend for cross-platform terminal handling. UI rendering is stateless and purely functional.

**Input Modes**: The application has distinct input modes (`Normal`, `Editing`, `AddingTask`, `MovingTask`) that completely change key binding behavior. Mode transitions are managed centrally in the App struct.

**Development Standards**: The codebase follows standard Rust conventions. Use `cargo fmt` to maintain consistent formatting and `cargo clippy` to catch common mistakes and improve code quality before committing changes.

## Critical Implementation Details

### Selection State Management

After loading the board from storage, selection indices (`selected_column`, `selected_task`) must be validated using `App::validate_selection()` to ensure they point to valid positions. This prevents navigation issues when the loaded board structure differs from the default state.

### Key Bindings Summary

**Normal Mode**:
- `hjkl` or Arrow keys: Navigate columns and tasks
- `n`: Add new task
- `Enter`: Edit selected task  
- `d`: Delete task
- `m`: Enter move mode, `M`: Move to previous column
- `q`: Quit

**Edit/Add Mode**:
- Type: Enter text for current field
- `Tab`, `↓`: Next field
- `↑`: Previous field
- `+`/`=`: Increase priority, `-`: Decrease priority
- `Enter`: Confirm, `Esc`: Cancel

**Move Mode**:
- `←`/`→`: Select target column
- `m`: Confirm move
- `Esc`: Cancel