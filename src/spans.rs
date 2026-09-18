use std::{fs, path::Path};

pub fn create_span(id: &str, version: &str,file: &str, start: usize, end: usize) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home.join(".lattice").join("documents").join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let metadata = match fs::read_to_string(document_dir.join("metadata")) {
        Ok(metadata) => metadata,
        Err(_) => {
            println!("metadata not found");
            return;
        }
    };

    let document_path = match metadata.lines().find_map(|line| line.strip_prefix("path=")) {
        Some(path) => Path::new(path),
        None => {
            println!("document path not found");
            return;
        }
    };

    let file_path = if version == "current"{
        document_path.join(file)
    } else {
        let version_path = document_dir
            .join("versions")
            .join(version);
        if !version_path.is_dir() {
            println!("vesion path not found: {}", version);
            return;
        }

        version_path.join(file)
    };

    let contents = match fs::read_to_string(&file_path) {
        Ok(contents) => contents,
        Err(_) => {
            println!("file not found: {}", file);
            return;
        }
    };

    if start > end || end > contents.len() {
        println!("invalid span range");
        return;
    }

    if !contents.is_char_boundary(start) || !contents.is_char_boundary(end) {
        println!("span must start and end on character boundaries");
        return;
    }

    let objects_dir = document_dir.join("objects");
    let spans_dir = document_dir.join("spans");

    fs::create_dir_all(&objects_dir).unwrap();
    fs::create_dir_all(&spans_dir).unwrap();

    let object_number = fs::read_dir(&objects_dir).unwrap().count() + 1;
    let span_number = fs::read_dir(&spans_dir).unwrap().count() + 1;

    let object_id = format!("O{}", object_number);
    let span_id = format!("S{}", span_number);

    for entry in fs::read_dir(&spans_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let existing_span = fs::read_to_string(path).unwrap();

        let mut existing_file = String::new();
        let mut existing_version = String::new();
        let mut existing_end = 0;
        let mut existing_start = 0;
        
        for line in existing_span.lines() {
             if line.starts_with("start=") {
                existing_start = line.strip_prefix("start=").unwrap().parse().unwrap();
                continue;
            } else if line.starts_with("end=") {
                existing_end = line.strip_prefix("end=").unwrap().parse().unwrap();
            } else if line.starts_with("file="){            
                existing_file = line.strip_prefix("file=").unwrap().to_string();
            } else if line.starts_with("version="){
                existing_version = line.strip_prefix("version=").unwrap().to_string();
            }
        }

        if existing_start == start
            && existing_end == end
            && existing_file == file
            && existing_version == version
        {
            println!("span already exists");
            return;
        }

    }

    fs::write(objects_dir.join(&object_id), &contents).unwrap();

    fs::write(
        spans_dir.join(&span_id),
        format!(
            "object={}\nfile={}\nstart={}\nend={}\nversion={}",
            object_id, file, start, end, version
        ),
    )
    .unwrap();

    println!("created {}", span_id);
}

pub fn remove_span(id: &str, span_id:&str){
    let home = dirs::home_dir().unwrap();

    let document_dir = home.join(".lattice").join("documents").join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let span_path = document_dir.join("spans").join(span_id); 

    let span = match fs::read_to_string(&span_path) {
        Ok(span) => span,
        Err(_) => {
            println!("span not found: {}", span_id);
            return;
        }
    };

    let mut object_id = String::new();

    for line in span.lines(){        
        if line.starts_with("object=") {
            object_id = line.strip_prefix("object=").unwrap().to_string();
            continue;
        }
    }
        
    let object_path = document_dir.join("objects").join(&object_id);

    
    match fs::remove_file(object_path) {
        Ok(_) => println!("object removed"),
        Err(_) =>println!("failed to remove the spans object")
    }
    
    match fs::remove_file(span_path) {
        Ok(_) => println!("span removed"),
        Err(_) =>println!("failed to remove span")
    }
}

pub fn show_span(
    id: &str,
    _version: &str,
    span_id: &str,
    all: bool,
){
    let home = dirs::home_dir().unwrap();

    let document_dir = home.join(".lattice").join("documents").join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let span_path = document_dir.join("spans").join(span_id);

    let span = match fs::read_to_string(&span_path) {
        Ok(span) => span,
        Err(_) => {
            println!("span not found: {}", span_id);
            return;
        }
    };
    
    let mut start = 0;
    let mut end = 0;
    let mut object_id = String::new();
    let mut file = String::new();
    let mut version = String::new();

    for line in span.lines() {
        if line.starts_with("object=") {
            object_id = line.strip_prefix("object=").unwrap().to_string();
            continue;
        } else if line.starts_with("start=") {
            start = line.strip_prefix("start=").unwrap().parse().unwrap();
        } else if line.starts_with("end=") {
            end = line.strip_prefix("end=").unwrap().parse().unwrap();
        } else if line.starts_with("file="){            
            file = line.strip_prefix("file=").unwrap().to_string();
        } else if line.starts_with("version=") {
            version = line.strip_prefix("version=").unwrap().to_string();
        }
    }

    let object_path = document_dir.join("objects").join(&object_id);

    let object = match fs::read_to_string(&object_path) {
        Ok(object) => object,
        Err(_) => {
            println!("object not found: {}", object_id);
            return;
        }
    };

    if all {
        println!("╭─ Span {}", span_id);
        println!("│");
        println!("│  Object:  {}", object_id);
        println!("│  Version: {}", version);
        println!("│  File:    {}", file);
        println!("│  Range:   {}..{}", start, end);
        println!("│");
        println!("│  Content:");
        println!("│  ─────────────────────────");
        println!("│  {}", object[start..end].trim_end());
        println!("│");
        println!("╰────────────────────────────────");
    } else {
        println!("{}", &object[start..end]);
    }

}

pub fn list_spans(id: &str, version_filter: Option<&str>) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let spans_dir = document_dir.join("spans");

    let entries = match fs::read_dir(&spans_dir) {
        Ok(entries) => entries,
        Err(_) => {
            println!("no spans found");
            return;
        }
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

        let span_id = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut object_id = String::new();
        let mut version = String::new();
        let mut file = String::new();
        let mut start = 0;
        let mut end = 0;

        for line in span.lines() {
            if let Some(value) = line.strip_prefix("object=") {
                object_id = value.to_string();
            } else if let Some(value) = line.strip_prefix("version=") {
                version = value.to_string();
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
            } else if let Some(value) = line.strip_prefix("file=") {
                file = value.to_string();
            }
        }

        if let Some(filter) = version_filter {
            if version != filter {
                continue;
            }
        }

        spans.push((
            span_id,
            object_id,
            version,
            file,
            start,
            end,
        ));
    }

    spans.sort_by(|a, b| {
        let a_num = a.0.trim_start_matches('S').parse::<usize>().unwrap_or(0);
        let b_num = b.0.trim_start_matches('S').parse::<usize>().unwrap_or(0);

        a_num.cmp(&b_num)
    });

    let title = match version_filter {
        Some(version) => format!("Spans for {} — {}", id, version),
        None => format!("Spans for {}", id),
    };

    println!();
    println!("╭─ {} ", title);
    println!("│");

    if spans.is_empty() {
        println!("│  No spans found.");
        println!("│");
        println!("╰────────────────────────────────────────────────────────");
        return;
    }

    println!(
        "│  {:<6} {:<8} {:<10} {:<24} {:>12}",
        "ID",
        "Object",
        "Version",
        "File",
        "Range"
    );

    println!(
        "│  ────── ──────── ────────── ──────────────────────── ────────────"
    );

    for (span_id, object_id, version, file, start, end) in spans {
        println!(
            "│  {:<6} {:<8} {:<10} {:<24} {:>5}..{:<5}",
            span_id,
            object_id,
            version,
            file,
            start,
            end
        );
    }

    println!("│");
    println!("╰────────────────────────────────────────────────────────");
}

pub fn gc(id: &str) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let spans_dir = document_dir.join("spans");
    let objects_dir = document_dir.join("objects");

    if !spans_dir.is_dir() || !objects_dir.is_dir() {
        println!("nothing to clean");
        return;
    }

    let mut referenced_objects = Vec::new();

    let entries = match fs::read_dir(&spans_dir) {
        Ok(entries) => entries,
        Err(_) => {
            println!("failed to read spans");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let span = match fs::read_to_string(&path) {
            Ok(span) => span,
            Err(_) => continue,
        };

        for line in span.lines() {
            if let Some(object_id) = line.strip_prefix("object=") {
                referenced_objects.push(object_id.to_string());
            }
        }
    }

    let objects = match fs::read_dir(&objects_dir) {
        Ok(objects) => objects,
        Err(_) => {
            println!("failed to read objects");
            return;
        }
    };

    let mut removed = 0;

    for entry in objects.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let object_id = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        if !referenced_objects.contains(&object_id) {
            match fs::remove_file(&path) {
                Ok(_) => {
                    println!("removed {}", object_id);
                    removed += 1;
                }

                Err(_) => {
                    println!("failed to remove {}", object_id);
                }
            }
        }
    }

    if removed == 0 {
        println!("nothing to clean");
    } else {
        println!("removed {} unused object(s)", removed);
    }
}
