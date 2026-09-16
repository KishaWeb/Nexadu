use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{fs, path::Path};

fn load_spans(
    document_dir: &Path,
    version: &str,
    file: &str,
) -> Vec<(usize, usize)> {
    let spans_dir = document_dir.join("spans");

    let entries = match fs::read_dir(&spans_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut spans = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let span = match fs::read_to_string(&path) {
            Ok(span) => span,
            Err(_) => continue,
        };

        let mut span_version = String::new();
        let mut span_file = String::new();
        let mut start = 0;
        let mut end = 0;

        for line in span.lines() {
            if let Some(value) = line.strip_prefix("version=") {
                span_version = value.to_string();
            } else if let Some(value) = line.strip_prefix("file=") {
                span_file = value.to_string();
            } else if let Some(value) = line.strip_prefix("start=") {
                start = match value.parse() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
            } else if let Some(value) = line.strip_prefix("end=") {
                end = match value.parse() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
            }
        }

        if span_version == version && span_file == file {
            spans.push((start, end));
        }
    }

    spans.sort_by_key(|(start, _)| *start);

    spans
}

fn highlight_spans(
    content: &str,
    spans: &[(usize, usize)],
) -> Text<'static> {
    let mut lines = Vec::new();
    let mut line_start = 0;

    for line in content.lines() {
        let line_end = line_start + line.len();
        let mut position = line_start;
        let mut line_spans = Vec::new();

        for &(start, end) in spans {
            if end <= line_start || start >= line_end {
                continue;
            }

            let span_start = start.max(line_start);
            let span_end = end.min(line_end);

            if position < span_start {
                line_spans.push(Span::raw(
                    content[position..span_start].to_string(),
                ));
            }

            line_spans.push(Span::styled(
                content[span_start..span_end].to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ));

            position = span_end;
        }

        if position < line_end {
            line_spans.push(Span::raw(
                content[position..line_end].to_string(),
            ));
        }

        lines.push(Line::from(line_spans));

        line_start = line_end + 1;
    }

    Text::from(lines)
}

pub fn tui(id: &str, version: Option<&str>) -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return Ok(());
    }

    let display_version = version.unwrap_or("current");

    let view_dir = match version {
        Some(version) => document_dir
            .join("versions")
            .join(version),

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

    files.sort();

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
                    .block(
                        Block::bordered()
                            .title(format!("Files — {}", display_version)),
                    )
                    .highlight_symbol("> ");

                frame.render_stateful_widget(
                    file_list,
                    left,
                    &mut state,
                );

                let content = if files.is_empty() {
                    String::from("No files")
                } else {
                    fs::read_to_string(&files[selected])
                        .unwrap_or_else(|_| {
                            String::from("Could not read file")
                        })
                };

                let title = files
                    .get(selected)
                    .and_then(|path| path.file_name())
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();

                let spans = load_spans(
                    &document_dir,
                    display_version,
                    &title,
                );

                let text = highlight_spans(&content, &spans);

                let right_panel = Paragraph::new(text)
                    .block(
                        Block::bordered()
                            .title(format!(
                                "{} — {}",
                                title,
                                display_version
                            )),
                    )
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0));

                frame.render_widget(right_panel, right);

                let guide_text = Paragraph::new(
                    " ↑↓ select file   j/k scroll   q quit ",
                );

                frame.render_widget(guide_text, guide);
            })?;

            if event::poll(
                std::time::Duration::from_millis(100),
            )? {
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
                                selected =
                                    (selected + 1).min(files.len() - 1);
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
