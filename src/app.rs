use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use crate::chezmoi;
use crate::utils::FileStatus;

#[derive(Debug)]
pub struct FileItem {
    pub(crate) path: String,
    pub(crate) selected: bool,
    pub(crate) local_status: FileStatus,
    pub(crate) source_status: FileStatus,
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    pub files: Vec<FileItem>,
    chezmoi_file_diff: String,
    list_state: ListState,
    error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            running: false,
            files: Vec::new(),
            chezmoi_file_diff: String::new(),
            list_state: ListState::default(),
            error_message: None,
        };
        app.files = chezmoi::update_status();
        app.list_state.select(Some(0));
        app.update_selected_diff();
        app
    }

    // --------------------------------------------------------
    // Helper methods
    // --------------------------------------------------------

    fn get_highlighted_file(&self) -> String {
        self.list_state
            .selected()
            .and_then(|i| self.files.get(i))
            .map(|file| file.path.clone())
            .unwrap_or_default()
    }

    fn get_selected_files(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.path.clone())
            .collect()
    }

    fn update_selected_diff(&mut self) {
        self.chezmoi_file_diff.clear();
        if let Some(selected) = self.list_state.selected() {
            if let Some(file) = self.files.get(selected) {
                self.chezmoi_file_diff = chezmoi::diff(&file.path);
            }
        }
    }

    // --------------------------------------------------------
    // Commands
    // --------------------------------------------------------

    fn toggle_selected_file(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(file) = self.files.get_mut(selected) {
                file.selected = !file.selected;
            }
        }
    }

    fn apply_selected_files(&mut self) {
        let selected_files = self.get_selected_files();
        if !selected_files.is_empty() {
            match chezmoi::apply(&selected_files) {
                Ok(_) => {
                    self.files = chezmoi::update_status();
                    self.update_selected_diff();
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(e.to_string());
                }
            }
        }
    }

    fn add_selected_files(&mut self) {
        let selected_files = self.get_selected_files();
        if !selected_files.is_empty() {
            match chezmoi::add(&selected_files) {
                Ok(_) => {
                    self.files = chezmoi::update_status();
                    self.update_selected_diff();
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(e.to_string());
                }
            }
        }
    }

    fn edit_highlighted_file(&mut self) {
        let highlighted_file = self.get_highlighted_file();
        if !highlighted_file.is_empty() {
            self.quit();
            chezmoi::edit(highlighted_file);
        }
    }

    fn open_chezmoi_source(&mut self) {
        self.quit();
        chezmoi::open_source();
    }

    fn quit(&mut self) {
        self.running = false;
    }

    // --------------------------------------------------------
    // UI
    // --------------------------------------------------------

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        terminal.clear()?;
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(frame.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[0]);

        let status_title = Line::from("Chezmoi Status").bold().blue().centered();
        let diff_title = Line::from("Chezmoi Diff").bold().blue().centered();

        // Status list rendering with selection indicators
        let items: Vec<ListItem> = self
            .files
            .iter()
            .map(|file| {
                let (local_symbol, local_style) = match file.local_status {
                    FileStatus::Added => ("A", Style::default().fg(Color::Green)),
                    FileStatus::Modified => ("M", Style::default().fg(Color::Yellow)),
                    FileStatus::Deleted => ("D", Style::default().fg(Color::Red)),
                    FileStatus::Untracked => ("?", Style::default().fg(Color::Red)),
                    FileStatus::Unchanged => (" ", Style::default()),
                };

                let (source_symbol, source_style) = match file.source_status {
                    FileStatus::Added => ("A", Style::default().fg(Color::Green)),
                    FileStatus::Modified => ("M", Style::default().fg(Color::Yellow)),
                    FileStatus::Deleted => ("D", Style::default().fg(Color::Red)),
                    FileStatus::Untracked => ("?", Style::default().fg(Color::Red)),
                    FileStatus::Unchanged => (" ", Style::default()),
                };

                let selection_prefix = if file.selected { "✓" } else { " " };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", selection_prefix),
                        if file.selected {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        },
                    ),
                    Span::styled(local_symbol, local_style),
                    Span::styled(source_symbol, source_style),
                    Span::raw(" "),
                    Span::raw(&file.path),
                ]))
            })
            .collect();

        frame.render_stateful_widget(
            List::new(items)
                .block(Block::bordered().title(status_title))
                .highlight_style(Style::default().bg(Color::DarkGray)),
            content_chunks[0],
            &mut self.list_state,
        );

        // Coloured diff rendering
        let diff_lines: Vec<Line> = self
            .chezmoi_file_diff
            .lines()
            .map(|line| {
                if line.starts_with('+') {
                    Line::from(line.to_string()).green()
                } else if line.starts_with('-') {
                    Line::from(line.to_string()).red()
                } else if line.starts_with("@@") {
                    Line::from(line.to_string()).cyan()
                } else {
                    Line::from(line.to_string())
                }
            })
            .collect();

        frame.render_widget(
            Paragraph::new(diff_lines).block(Block::bordered().title(diff_title)),
            content_chunks[1],
        );

        // Add help/Error message section at the bottom
        if let Some(error) = &self.error_message {
            let error_text = Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red)),
                Span::raw(error),
            ]);

            frame.render_widget(
                Paragraph::new(error_text)
                    .style(Style::default().fg(Color::Red))
                    .alignment(ratatui::layout::Alignment::Left),
                main_chunks[1], // Use the bottom section where help text is
            );
        } else {
            let help_text = vec![
                "q/Esc".blue().bold(),
                " Quit".gray(),
                " | ".dark_gray(),
                "↑/k".blue().bold(),
                " Up".gray(),
                " | ".dark_gray(),
                "↓/j".blue().bold(),
                " Down".gray(),
                " | ".dark_gray(),
                "<space>".blue().bold(),
                " Toggle".gray(),
                " | ".dark_gray(),
                "e".blue().bold(),
                " Edit highlighted file".gray(),
                " | ".dark_gray(),
                "a".blue().bold(),
                " Add/re-add selected".gray(),
                " | ".dark_gray(),
                "A".blue().bold(),
                " Apply selected".gray(),
                " | ".dark_gray(),
                "S".blue().bold(),
                " Open chezmoi source".gray(),
            ];

            frame.render_widget(
                Paragraph::new(Line::from(help_text)).alignment(ratatui::layout::Alignment::Left),
                main_chunks[1],
            );
        }
    }

    // --------------------------------------------------------
    // Input event handling
    // --------------------------------------------------------

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char(' ')) => self.toggle_selected_file(),
            (_, KeyCode::Char('S')) => {
                self.open_chezmoi_source();
            }
            (_, KeyCode::Char('a')) => self.add_selected_files(),
            (_, KeyCode::Char('A')) => self.apply_selected_files(),
            (_, KeyCode::Char('e')) => self.edit_highlighted_file(),
            (_, KeyCode::Up | KeyCode::Char('k')) => self.previous_item(),
            (_, KeyCode::Down | KeyCode::Char('j')) => self.next_item(),
            _ => {}
        }
    }

    fn next_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_selected_diff();
    }

    fn previous_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_selected_diff();
    }
}
