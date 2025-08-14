use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, EditField, InputMode},
    models::Priority,
};

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(format!("Kanban TUI - {}", app.board.title))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()),
        );
    f.render_widget(header, chunks[0]);

    // Main kanban board
    render_board(f, chunks[1], app);

    // Status bar
    let status_text = match app.input_mode {
        InputMode::Normal => format!(
            "Status: {} | Controls: hjkl/arrows=move, n=new task, Enter=edit, d=delete, m/M=move task, q=quit",
            app.status_message
        ),
        InputMode::AddingTask => "Adding task - Enter: confirm, Esc: cancel, Tab/↓: next field, ↑: prev field".to_string(),
        InputMode::Editing => "Editing task - Enter: confirm, Esc: cancel, Tab/↓: next field, ↑: prev field".to_string(),
        InputMode::MovingTask => "Moving task - ←/→: select target column, Enter: confirm, Esc: cancel".to_string(),
    };

    let status_bar = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()),
        );
    f.render_widget(status_bar, chunks[2]);

    // Render input popup if needed
    match app.input_mode {
        InputMode::AddingTask | InputMode::Editing => render_input_popup(f, app),
        _ => {}
    }
}

fn render_board(f: &mut Frame, area: Rect, app: &App) {
    let column_width = area.width / app.board.columns.len() as u16;
    let columns_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            app.board
                .columns
                .iter()
                .map(|_| Constraint::Length(column_width))
                .collect::<Vec<_>>(),
        )
        .split(area);

    for (col_idx, column) in app.board.columns.iter().enumerate() {
        let is_selected_column = col_idx == app.selected_column;
        let is_target_column = matches!(app.input_mode, InputMode::MovingTask) && col_idx == app.target_column;
        
        let border_style = if is_target_column {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else if is_selected_column {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(format!("{} ({})", column.title, column.tasks.len()))
            .borders(Borders::ALL)
            .style(border_style);

        let tasks: Vec<ListItem> = column
            .tasks
            .iter()
            .enumerate()
            .map(|(task_idx, task)| {
                let is_selected = is_selected_column && task_idx == app.selected_task;
                let is_being_moved = app.moving_task_id.is_some() && app.moving_task_id == Some(task.id);
                
                let priority_indicator = match task.priority {
                    Priority::Critical => "🔴",
                    Priority::High => "🟡",
                    Priority::Medium => "🔵",
                    Priority::Low => "🟢",
                };

                let due_date_str = task
                    .due_date
                    .map(|date| format!(" [{}]", date.format("%m/%d")))
                    .unwrap_or_default();

                let content = format!("{} {}{}", priority_indicator, task.title, due_date_str);
                
                let style = if is_being_moved {
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(tasks).block(block);
        
        if let Some(column_area) = columns_layout.get(col_idx) {
            f.render_widget(list, *column_area);
        }
    }
}

fn render_input_popup(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, popup_area);

    let title = match app.input_mode {
        InputMode::AddingTask => "Add New Task",
        InputMode::Editing => "Edit Task",
        _ => "Input",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title input
            Constraint::Length(5), // Description input
            Constraint::Length(3), // Priority selection
            Constraint::Min(0),     // Spacer
        ])
        .margin(1)
        .split(inner);

    // Title input
    let title_selected = matches!(app.edit_state.selected_field, EditField::Title);
    let title_style = if title_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let title_border_style = if title_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let title_input = Paragraph::new(app.edit_state.title.as_str())
        .style(title_style)
        .block(
            Block::default()
                .title(if title_selected { "Title [SELECTED]" } else { "Title" })
                .borders(Borders::ALL)
                .style(title_border_style),
        );
    f.render_widget(title_input, input_chunks[0]);

    // Description input
    let desc_selected = matches!(app.edit_state.selected_field, EditField::Description);
    let desc_style = if desc_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_border_style = if desc_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let description_input = Paragraph::new(app.edit_state.description.as_str())
        .style(desc_style)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(if desc_selected { "Description [SELECTED]" } else { "Description" })
                .borders(Borders::ALL)
                .style(desc_border_style),
        );
    f.render_widget(description_input, input_chunks[1]);

    // Priority selection
    let priority_selected = matches!(app.edit_state.selected_field, EditField::Priority);
    let priority_text = format!("Priority: {}", app.edit_state.priority);
    let priority_border_style = if priority_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    let priority_display = Paragraph::new(priority_text)
        .style(Style::default())
        .block(
            Block::default()
                .title(if priority_selected { "Priority (use +/- to change) [SELECTED]" } else { "Priority (use +/- to change)" })
                .borders(Borders::ALL)
                .style(priority_border_style),
        );
    f.render_widget(priority_display, input_chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}