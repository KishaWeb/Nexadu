use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
struct Reference {
    id: String,
    start: usize,
    end: usize,
    to_document: String,
    to_version: String,
    to_span: String,
}

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
        let mut start = None;
        let mut end = None;

        for line in span.lines() {
            if let Some(value) = line.strip_prefix("version=") {
                span_version = value.to_string();
            } else if let Some(value) = line.strip_prefix("file=") {
                span_file = value.to_string();
            } else if let Some(value) = line.strip_prefix("start=") {
                if let Ok(value) = value.parse::<usize>() {
                    start = Some(value);
                }
            } else if let Some(value) = line.strip_prefix("end=") {
                if let Ok(value) = value.parse::<usize>() {
                    end = Some(value);
                }
            }
        }

        if span_version == version && span_file == file {
            if let (Some(start), Some(end)) = (start, end) {
                if start < end {
                    spans.push((start, end));
                }
            }
        }
    }

    spans.sort_unstable_by_key(|(start, _)| *start);
    spans
}

fn load_references(
    document_dir: &Path,
    document_id: &str,
    version: &str,
    file: &str,
) -> Vec<Reference> {
    let references_dir = document_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("references");

    let entries = match fs::read_dir(&references_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut references = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let reference = match fs::read_to_string(&path) {
            Ok(reference) => reference,
            Err(_) => continue,
        };

        let reference_id = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        let mut from_document = String::new();
        let mut from_version = String::new();
        let mut from_span = String::new();

        let mut to_document = String::new();
        let mut to_version = String::new();
        let mut to_span = String::new();

        for line in reference.lines() {
            if let Some(value) = line.strip_prefix("from_document=") {
                from_document = value.to_string();
            } else if let Some(value) = line.strip_prefix("from_version=") {
                from_version = value.to_string();
            } else if let Some(value) = line.strip_prefix("from_span=") {
                from_span = value.to_string();
            } else if let Some(value) = line.strip_prefix("to_document=") {
                to_document = value.to_string();
            } else if let Some(value) = line.strip_prefix("to_version=") {
                to_version = value.to_string();
            } else if let Some(value) = line.strip_prefix("to_span=") {
                to_span = value.to_string();
            }
        }

        if from_document != document_id || from_version != version {
            continue;
        }

        let span_path = document_dir.join("spans").join(&from_span);

        let span = match fs::read_to_string(&span_path) {
            Ok(span) => span,
            Err(_) => continue,
        };

        let mut span_version = String::new();
        let mut span_file = String::new();
        let mut start = None;
        let mut end = None;

        for line in span.lines() {
            if let Some(value) = line.strip_prefix("version=") {
                span_version = value.to_string();
            } else if let Some(value) = line.strip_prefix("file=") {
                span_file = value.to_string();
            } else if let Some(value) = line.strip_prefix("start=") {
                if let Ok(value) = value.parse::<usize>() {
                    start = Some(value);
                }
            } else if let Some(value) = line.strip_prefix("end=") {
                if let Ok(value) = value.parse::<usize>() {
                    end = Some(value);
                }
            }
        }

        if span_version != version || span_file != file {
            continue;
        }

        let (start, end) = match (start, end) {
            (Some(start), Some(end)) if start < end => (start, end),
            _ => continue,
        };

        references.push(Reference {
            id: reference_id,
            start,
            end,
            to_document,
            to_version,
            to_span,
        });
    }

    references.sort_by(|a, b| {
        let a_num = a
            .id
            .trim_start_matches('R')
            .parse::<usize>()
            .unwrap_or(0);

        let b_num = b
            .id
            .trim_start_matches('R')
            .parse::<usize>()
            .unwrap_or(0);

        a_num.cmp(&b_num)
    });

    references
}

fn highlight_spans(
    content: &str,
    spans: &[(usize, usize)],
    references: &[Reference],
    selected_reference: Option<usize>,
    show_spans: bool,
    show_references: bool,
) -> Text<'static> {
    let content_len = content.len();
    let mut lines = Vec::new();
    let mut line_start = 0;

    for line in content.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        let line_end = line_start + line_without_newline.len();

        let mut boundaries = vec![line_start, line_end];

        if show_spans {
            for &(start, end) in spans {
                let start = start.min(content_len);
                let end = end.min(content_len);

                if start < end && end > line_start && start < line_end {
                    boundaries.push(start.max(line_start));
                    boundaries.push(end.min(line_end));
                }
            }
        }

        if show_references {
            for reference in references {
                let start = reference.start.min(content_len);
                let end = reference.end.min(content_len);

                if start < end && end > line_start && start < line_end {
                    boundaries.push(start.max(line_start));
                    boundaries.push(end.min(line_end));
                }
            }
        }

        boundaries.sort_unstable();
        boundaries.dedup();

        let mut line_spans = Vec::new();

        for window in boundaries.windows(2) {
            let start = window[0];
            let end = window[1];

            if start >= end || start > content_len || end > content_len {
                continue;
            }

            if !content.is_char_boundary(start) || !content.is_char_boundary(end) {
                continue;
            }

            let inside_span = show_spans
                && spans.iter().any(|&(span_start, span_end)| {
                    start >= span_start && end <= span_end
                });

            let reference_index = if show_references {
                references
                    .iter()
                    .enumerate()
                    .filter(|(_, reference)| {
                        start >= reference.start && end <= reference.end
                    })
                    .map(|(index, _)| index)
                    .next()
            } else {
                None
            };

            let mut style = Style::default();

            if inside_span {
                style = style.add_modifier(Modifier::DIM);
            }

            if reference_index.is_some() {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            if let Some(index) = reference_index {
                if selected_reference == Some(index) {
                    style = style.add_modifier(Modifier::REVERSED);
                }
            }
            
            let text = content[start..end].to_string();

            if style == Style::default() {
                line_spans.push(Span::raw(text));
            } else {
                line_spans.push(Span::styled(text, style));
            }
        }

        if line_spans.is_empty() {
            line_spans.push(Span::raw(line_without_newline.to_string()));
        }

        lines.push(Line::from(line_spans));

        line_start += line.len();
    }

    if content.is_empty() {
        lines.push(Line::from(""));
    }

    Text::from(lines)
}

fn get_view_dir(
    document_dir: &Path,
    version: &str,
) -> Option<PathBuf> {
    if version == "current" {
        let metadata = fs::read_to_string(document_dir.join("metadata"))
            .ok()?;

        let path = metadata
            .lines()
            .find_map(|line| line.strip_prefix("path="))?;

        let path = PathBuf::from(path);

        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    } else {
        let path = document_dir
            .join("versions")
            .join(version);

        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    }
}

fn get_files(view_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(view_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        }
    }

    files.sort();

    Ok(files)
}

fn target_file(
    Nexadu_dir: &Path,
    document: &str,
    version: &str,
    span_id: &str,
) -> Option<String> {
    let span_path = Nexadu_dir
        .join("documents")
        .join(document)
        .join("spans")
        .join(span_id);

    let span = fs::read_to_string(span_path).ok()?;

    let mut span_version = String::new();
    let mut file = String::new();

    for line in span.lines() {
        if let Some(value) = line.strip_prefix("version=") {
            span_version = value.to_string();
        } else if let Some(value) = line.strip_prefix("file=") {
            file = value.to_string();
        }
    }

    if span_version != version {
        return None;
    }

    Some(file)
}

pub fn tui(id: &str, version: Option<&str>) -> std::io::Result<()> {
    let home = dirs::home_dir().unwrap();
    let Nexadu_dir = home.join(".Nexadu");

    let mut current_id = id.to_string();
    let mut current_version =
        version.unwrap_or("current").to_string();

    let mut selected = 0;
    let mut scroll = 0;

    let mut show_spans = false;
    let mut show_references = true;

    let mut reference_index: Option<usize> = None;

    let mut state = ListState::default();

    ratatui::run(|terminal| {
        loop {
            let document_dir = Nexadu_dir
                .join("documents")
                .join(&current_id);

            if !document_dir.is_dir() {
                break;
            }

            let view_dir = match get_view_dir(
                &document_dir,
                &current_version,
            ) {
                Some(path) => path,
                None => break,
            };

            let files = match get_files(&view_dir) {
                Ok(files) => files,
                Err(_) => break,
            };

            if files.is_empty() {
                selected = 0;
                state.select(None);
            } else {
                selected = selected.min(files.len() - 1);
                state.select(Some(selected));
            }

            let title = files
                .get(selected)
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();

            let content = if files.is_empty() {
                String::from("No files")
            } else {
                fs::read_to_string(&files[selected])
                    .unwrap_or_else(|_| {
                        String::from("Could not read file")
                    })
            };

            let spans = load_spans(
                &document_dir,
                &current_version,
                &title,
            );

            let references = load_references(
                &document_dir,
                &current_id,
                &current_version,
                &title,
            );

            if let Some(index) = reference_index {
                if index >= references.len() {
                    reference_index = None;
                }
            }

            terminal.draw(|frame| {
                let [main, reference_panel, guide] =
                    Layout::vertical([
                        Constraint::Min(1),
                        Constraint::Length(
                            if reference_index.is_some() {
                                3
                            } else {
                                1
                            },
                        ),
                        Constraint::Length(1),
                    ])
                    .areas(frame.area());

                let [left, right] =
                    Layout::horizontal([
                        Constraint::Percentage(25),
                        Constraint::Percentage(75),
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
                            .title(format!(
                                "Files — {}",
                                current_version
                            )),
                    )
                    .highlight_symbol("> ");

                frame.render_stateful_widget(
                    file_list,
                    left,
                    &mut state,
                );

                let text = highlight_spans(
                    &content,
                    &spans,
                    &references,
                    reference_index,
                    show_spans,
                    show_references,
                );

                let right_panel = Paragraph::new(text)
                    .block(
                        Block::bordered()
                            .title(format!(
                                "{} — {}",
                                title,
                                current_version
                            )),
                    )
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0));

                frame.render_widget(
                    right_panel,
                    right,
                );

                if let Some(index) = reference_index {
                    if let Some(reference) =
                        references.get(index)
                    {
                        let reference_text = format!(
                            " {}  →  {}:{}:{}    Enter: follow",
                            reference.id,
                            reference.to_document,
                            reference.to_version,
                            reference.to_span,
                        );

                        let panel =
                            Paragraph::new(reference_text)
                                .block(
                                    Block::bordered()
                                        .title("Reference"),
                                );

                        frame.render_widget(
                            panel,
                            reference_panel,
                        );
                    }
                }

                let guide_text = Paragraph::new(
                    " ↑↓ files   j/k scroll   Tab refs   Enter follow   s spans   r refs   q quit ",
                );

                frame.render_widget(
                    guide_text,
                    guide,
                );
            })?;

            if event::poll(
                std::time::Duration::from_millis(100),
            )? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,

                        KeyCode::Up => {
                            if !files.is_empty() {
                                selected =
                                    selected.saturating_sub(1);

                                scroll = 0;
                                reference_index = None;

                                state.select(Some(selected));
                            }
                        }

                        KeyCode::Down => {
                            if !files.is_empty() {
                                selected =
                                    (selected + 1)
                                        .min(files.len() - 1);

                                scroll = 0;
                                reference_index = None;

                                state.select(Some(selected));
                            }
                        }

                        KeyCode::Char('j') => {
                            scroll = scroll.saturating_add(1);
                        }

                        KeyCode::Char('k') => {
                            scroll = scroll.saturating_sub(1);
                        }

                        KeyCode::Char('s') => {
                            show_spans = !show_spans;
                        }

                        KeyCode::Char('r') => {
                            show_references =
                                !show_references;

                            if !show_references {
                                reference_index = None;
                            }
                        }

                        KeyCode::Tab => {
                            if !references.is_empty() {
                                reference_index =
                                    Some(match reference_index {
                                        Some(index) => {
                                            (index + 1)
                                                % references.len()
                                        }
                                        None => 0,
                                    });
                            }
                        }

                        KeyCode::BackTab => {
                            if !references.is_empty() {
                                reference_index =
                                    Some(match reference_index {
                                        Some(0) | None => {
                                            references.len() - 1
                                        }
                                        Some(index) => {
                                            index - 1
                                        }
                                    });
                            }
                        }

                        KeyCode::Enter => {
                            if let Some(index) =
                                reference_index
                            {
                                if let Some(reference) =
                                    references.get(index)
                                {
                                    if let Some(file) =
                                        target_file(
                                            &Nexadu_dir,
                                            &reference.to_document,
                                            &reference.to_version,
                                            &reference.to_span,
                                        )
                                    {
                                        let target_dir =
                                            Nexadu_dir
                                                .join("documents")
                                                .join(&reference.to_document);

                                        let target_view =
                                            get_view_dir(
                                                &target_dir,
                                                &reference.to_version,
                                            );

                                        if let Some(target_view) =
                                            target_view
                                        {
                                            let target_files =
                                                get_files(
                                                    &target_view,
                                                )?;

                                            if let Some(target_index) =
                                                target_files
                                                    .iter()
                                                    .position(|path| {
                                                        path.file_name()
                                                            .map(|name| {
                                                                name.to_string_lossy()
                                                                    == file
                                                            })
                                                            .unwrap_or(false)
                                                    })
                                            {
                                                current_id =
                                                    reference
                                                        .to_document
                                                        .clone();

                                                current_version =
                                                    reference
                                                        .to_version
                                                        .clone();

                                                selected =
                                                    target_index;

                                                scroll = 0;
                                                reference_index =
                                                    None;

                                                state.select(
                                                    Some(selected),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(())
    })
}
