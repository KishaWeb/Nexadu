use std::{fs, path::Path};

pub fn create_span(id: &str, file: &str, start: usize, end: usize) {
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

    let file_path = document_path.join(file);

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

        let mut state = false;

        for line in existing_span.lines() {
            if line.starts_with("object=") {
                state = true;
            } else if line.starts_with("start=") {
                state = true;
            } else if line.starts_with("end=") {
                state = true;
            }
        }

        if state == true {
            println!("span already exists");
            return;
        } else {
            continue;
        }
    }

    fs::write(objects_dir.join(&object_id), &contents).unwrap();

    fs::write(
        spans_dir.join(&span_id),
        format!(
            "object={}\nfile={}\nstart={}\nend={}",
            object_id, file, start, end
        ),
    )
    .unwrap();

    println!("created {}", span_id);
}

pub fn show_span(id: &str, span_id: &str) {
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

    for line in span.lines() {
        if line.starts_with("object=") {
            object_id = line.strip_prefix("object=").unwrap().to_string();
            continue;
        } else if line.starts_with("start=") {
            start = line.strip_prefix("start=").unwrap().parse().unwrap();
        } else if line.starts_with("end=") {
            end = line.strip_prefix("end=").unwrap().parse().unwrap();
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

    println!("{}", &object[start..end]);
}
