use std::{fs, path::Path};

pub fn create_span(id: &str, file: &str) {
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

    let objects_dir = document_dir.join("objects");
    let spans_dir = document_dir.join("spans");

    fs::create_dir_all(&objects_dir).unwrap();
    fs::create_dir_all(&spans_dir).unwrap();

    let object_number = fs::read_dir(&objects_dir).unwrap().count() + 1;
    let span_number = fs::read_dir(&spans_dir).unwrap().count() + 1;

    let object_id = format!("O{}", object_number);
    let span_id = format!("S{}", span_number);

    fs::write(objects_dir.join(&object_id), contents).unwrap();

    fs::write(spans_dir.join(&span_id), format!("{}|{}", object_id, file)).unwrap();

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

    let object_id = match span.split('|').next() {
        Some(object_id) => object_id,
        None => {
            println!("invalid span");
            return;
        }
    };

    let object_path = document_dir.join("objects").join(object_id);

    let contents = match fs::read_to_string(object_path) {
        Ok(contents) => contents,
        Err(_) => {
            println!("object not found: {}", object_id);
            return;
        }
    };

    println!("Span: {}", span_id);
    println!();
    print!("{}", contents);
}
