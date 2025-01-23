use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Borders;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use crate::chezmoi;
use crate::utils::FileStatus;

#[derive(Debug, Clone)]
pub enum PopupAction {
    Apply,
    ReAdd,
    Cancel,
}

#[derive(Debug, Default, PartialEq)]
pub enum Selection {
    #[default]
    None,
    Source,
    Local,
}

#[derive(Debug)]
pub struct FileItem {
    pub(crate) path: String,
    pub(crate) selected: Selection,
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
    show_popup: bool,
    popup_items: Vec<(String, PopupAction)>, // Tuple of display string and action
    popup_state: ListState,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            running: false,
            files: Vec::new(),
            chezmoi_file_diff: String::new(),
            list_state: ListState::default(),
            error_message: None,
            show_popup: false,
            popup_items: Vec::new(),
            popup_state: ListState::default(),
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

    fn get_selected_local_files(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| {
                f.selected == Selection::Local
                    && matches!(
                        f.local_status,
                        FileStatus::Added
                            | FileStatus::Modified
                            | FileStatus::Deleted
                            | FileStatus::Untracked
                    )
            })
            .map(|f| f.path.clone())
            .collect()
    }

    fn get_selected_source_files(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| {
                f.selected == Selection::Source
                    && matches!(
                        f.source_status,
                        FileStatus::Added
                            | FileStatus::Modified
                            | FileStatus::Deleted
                            | FileStatus::Untracked
                    )
            })
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
                file.selected = match file.selected {
                    Selection::None => {
                        if file.local_status != FileStatus::Unchanged {
                            Selection::Local
                        } else {
                            Selection::Source
                        }
                    }
                    Selection::Local => {
                        if file.source_status != FileStatus::Unchanged {
                            Selection::Source
                        } else {
                            Selection::None
                        }
                    }
                    Selection::Source => Selection::None,
                };
            }
        }
    }

    fn apply_selected_files(&mut self) {
        let selected_files = self.get_selected_source_files();
        if !selected_files.is_empty() {
            match chezmoi::apply(&selected_files) {
                Ok(_) => {
                    for file in &mut self.files {
                        file.selected = Selection::None;
                    }
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

    fn re_add_selected_files(&mut self) {
        let selected_files = self.get_selected_local_files();
        if !selected_files.is_empty() {
            match chezmoi::re_add(&selected_files) {
                Ok(_) => {
                    for file in &mut self.files {
                        file.selected = Selection::None;
                    }
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

    fn draw_popup(&mut self, frame: &mut Frame) {
        let block = Block::default()
            .title("Select an action")
            .borders(Borders::ALL);
        let area = frame.area();

        let popup_area = Rect {
            x: (area.width - 60) / 2,
            y: (area.height - 10) / 2,
            width: 60,
            height: 10,
        };

        frame.render_widget(Clear, popup_area);

        // Update to use only the first part of the tuple (the display string)
        let items: Vec<ListItem> = self
            .popup_items
            .iter()
            .map(|(text, _)| ListItem::new(text.as_str()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, popup_area, &mut self.popup_state);
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

                let selection_prefix = match file.selected {
                    Selection::None => " ",
                    Selection::Local => "L",
                    Selection::Source => "S",
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", selection_prefix),
                        if file.selected == Selection::None {
                            Style::default()
                        } else {
                            Style::default().fg(Color::Green)
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
                " Select file(s)".gray(),
                " | ".dark_gray(),
                "E".blue().bold(),
                " Edit highlighted file in source".gray(),
                " | ".dark_gray(),
                "A".blue().bold(),
                " Apply/Re-add selected files".gray(),
                " | ".dark_gray(),
                "S".blue().bold(),
                " Open chezmoi source".gray(),
            ];

            frame.render_widget(
                Paragraph::new(Line::from(help_text)).alignment(ratatui::layout::Alignment::Left),
                main_chunks[1],
            );
        }

        if self.show_popup {
            self.draw_popup(frame);
        }
    }

    pub fn show_popup(&mut self, items: Vec<(String, PopupAction)>) {
        self.show_popup = true;
        self.popup_items = items;
        self.popup_state.select(Some(0));
    }

    pub fn show_action_popup(&mut self) {
        self.popup_items = vec![
            ("Apply selected files".to_string(), PopupAction::Apply),
            ("Re-add selected files".to_string(), PopupAction::ReAdd),
            ("Cancel".to_string(), PopupAction::Cancel),
        ];
        self.show_popup = true;
        self.popup_state.select(Some(0));
    }

    fn handle_popup_selection(&mut self) {
        if let Some(i) = self.popup_state.selected() {
            if let Some((_, action)) = self.popup_items.get(i) {
                match action {
                    PopupAction::Apply => self.apply_selected_files(),
                    PopupAction::ReAdd => self.re_add_selected_files(),
                    PopupAction::Cancel => self.show_popup = false,
                }
            }
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
        if self.show_popup {
            match key.code {
                KeyCode::Esc => {
                    self.show_popup = false;
                }
                KeyCode::Enter => {
                    self.handle_popup_selection();
                    self.show_popup = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = match self.popup_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.popup_items.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.popup_state.select(Some(i));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = match self.popup_state.selected() {
                        Some(i) => {
                            if i >= self.popup_items.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.popup_state.select(Some(i));
                }
                _ => {}
            }
        } else {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
                (_, KeyCode::Char(' ')) => self.toggle_selected_file(),
                (_, KeyCode::Char('S')) => {
                    self.open_chezmoi_source();
                }
                (_, KeyCode::Char('A')) => self.show_action_popup(),
                //(_, KeyCode::Char('A')) => self.apply_selected_files(),
                (_, KeyCode::Char('e')) => self.edit_highlighted_file(),
                (_, KeyCode::Up | KeyCode::Char('k')) => self.previous_item(),
                (_, KeyCode::Down | KeyCode::Char('j')) => self.next_item(),
                _ => {}
            }
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
