use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{
        Block, List, ListItem, ListState, Paragraph, Wrap,
    },
};
use std::{fs, path::Path};

pub fn tui(id: &str, version: Option<&str>) -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap();

    let document_dir = home.join(".lattice").join("documents").join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return Ok(());
    }

    let view_dir = match version {
        Some(version) => document_dir.join("versions").join(version),

        None => match fs::read_to_string(document_dir.join("metadata")) {
            Ok(metadata) => {
                let path = metadata
                    .lines()
                    .find_map(|line| line.strip_prefix("path="))
                    .map(Path::new);

                match path {
                    Some(path) if path.is_dir() => path.to_path_buf(),

                    _ => {
                        println!("document path not found");
                        return Ok(());
                    }
                }
            }

            Err(_) => {
                println!("metadata not found");
                return Ok(());
            }
        },
    };

    if !view_dir.is_dir() {
        println!("view directory not found");
        return Ok(());
    }

    let mut files = Vec::new();

    for entry in fs::read_dir(&view_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        }
    }

    let mut selected = 0;
    let mut scroll = 0;
    let mut state = ListState::default();

    if !files.is_empty() {
        state.select(Some(selected));
    }

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let [main, guide] = Layout::vertical([
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .areas(frame.area());

                let [left, right] = Layout::horizontal([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .areas(main);

                let items: Vec<ListItem> = files
                    .iter()
                    .map(|path| {
                        ListItem::new(
                            path.file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        )
                    })
                    .collect();

                let file_list = List::new(items)
                    .block(Block::bordered().title("Files"))
                    .highlight_symbol("> ");

                frame.render_stateful_widget(file_list, left, &mut state);

                let content = if files.is_empty() {
                    String::from("No files")
                } else {
                    fs::read_to_string(&files[selected])
                        .unwrap_or_else(|_| String::from("Could not read file"))
                };

                let title = files
                    .get(selected)
                    .and_then(|path| path.file_name())
                    .map(|name| name.to_string_lossy())
                    .unwrap_or_default();

                let right_panel = Paragraph::new(content)
                    .block(Block::bordered().title(title))
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0));

                frame.render_widget(right_panel, right);

                let guide_text = Paragraph::new(
                    " ↑↓ select file   j/k scroll   q quit ",
                );

                frame.render_widget(guide_text, guide);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,

                        KeyCode::Up => {
                            if !files.is_empty() {
                                selected = selected.saturating_sub(1);
                                scroll = 0;
                                state.select(Some(selected));
                            }
                        }

                        KeyCode::Down => {
                            if !files.is_empty() {
                                selected = (selected + 1).min(files.len() - 1);
                                scroll = 0;
                                state.select(Some(selected));
                            }
                        }

                        KeyCode::Char('j') => {
                            scroll = scroll.saturating_add(1);
                        }

                        KeyCode::Char('k') => {
                            scroll = scroll.saturating_sub(1);
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(())
    })
}
