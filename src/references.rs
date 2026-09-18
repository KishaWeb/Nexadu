use std::fs;

pub fn create_reference(
    from_document: &str,
    from_version: &str,
    from_span: &str,
    to_document: &str,
    to_version: &str,
    to_span: &str,
) {
    let home = dirs::home_dir().unwrap();
    let lattice_dir = home.join(".lattice");
    let references_dir = lattice_dir.join("references");

    fs::create_dir_all(&references_dir).unwrap();

    for document in [from_document, to_document] {
        let document_dir = lattice_dir.join("documents").join(document);

        if !document_dir.is_dir() {
            println!("document not found: {}", document);
            return;
        }
    }

    for (document, span) in [(from_document, from_span), (to_document, to_span)] {
        let span_path = lattice_dir
            .join("documents")
            .join(document)
            .join("spans")
            .join(span);

        if !span_path.is_file() {
            println!("span not found: {}:{}", document, span);
            return;
        }
    }

    let mut next_id = 1;

    if let Ok(entries) = fs::read_dir(&references_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            if let Some(number) = name.strip_prefix('R') {
                if let Ok(number) = number.parse::<usize>() {
                    next_id = next_id.max(number + 1);
                }
            }
        }
    }

    let reference_id = format!("R{}", next_id);
    let reference_path = references_dir.join(&reference_id);

    let metadata = format!(
        "from_document={}\n\
         from_version={}\n\
         from_span={}\n\
         to_document={}\n\
         to_version={}\n\
         to_span={}\n",
        from_document,
        from_version,
        from_span,
        to_document,
        to_version,
        to_span,
    );

    fs::write(&reference_path, metadata).unwrap();

    println!(
        "{}: {}:{}:{} → {}:{}:{}",
        reference_id,
        from_document,
        from_version,
        from_span,
        to_document,
        to_version,
        to_span
    );
}

pub fn show_reference(_id: &str, reference_id: &str) {
    let home = dirs::home_dir().unwrap();
    let lattice_dir = home.join(".lattice");

    let reference_path = lattice_dir
        .join("references")
        .join(reference_id);

    let reference = match fs::read_to_string(&reference_path) {
        Ok(reference) => reference,
        Err(_) => {
            println!("reference not found: {}", reference_id);
            return;
        }
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

    println!("╭─ Reference {}", reference_id);
    println!("│");
    println!(
        "│  From: {}:{}:{}",
        from_document,
        from_version,
        from_span
    );
    println!(
        "│  To:   {}:{}:{}",
        to_document,
        to_version,
        to_span
    );
    println!("│");

    for (label, document, version, span_id) in [
        ("From", &from_document, &from_version, &from_span),
        ("To", &to_document, &to_version, &to_span),
    ] {
        let span_path = lattice_dir
            .join("documents")
            .join(document)
            .join("spans")
            .join(span_id);

        let span = match fs::read_to_string(&span_path) {
            Ok(span) => span,
            Err(_) => {
                println!(
                    "│  {} span not found: {}:{}:{}",
                    label,
                    document,
                    version,
                    span_id
                );
                continue;
            }
        };

        let mut object_id = String::new();
        let mut start = 0;
        let mut end = 0;
        let mut span_version = String::new();

        for line in span.lines() {
            if let Some(value) = line.strip_prefix("object=") {
                object_id = value.to_string();
            } else if let Some(value) = line.strip_prefix("start=") {
                start = value.parse().unwrap();
            } else if let Some(value) = line.strip_prefix("end=") {
                end = value.parse().unwrap();
            } else if let Some(value) = line.strip_prefix("version=") {
                span_version = value.to_string();
            }
        }

        if span_version != *version {
            println!(
                "│  {} span version mismatch: expected {}, found {}",
                label,
                version,
                span_version
            );
            continue;
        }

        let object_path = lattice_dir
            .join("documents")
            .join(document)
            .join("objects")
            .join(&object_id);

        let object = match fs::read_to_string(&object_path) {
            Ok(object) => object,
            Err(_) => {
                println!("│  object not found: {}", object_id);
                continue;
            }
        };

        println!(
            "│  {} {}:{}:{}",
            label,
            document,
            version,
            span_id
        );
        println!("│  ─────────────────────────");
        println!("│  {}", &object[start..end]);
        println!("│");
    }

    println!("╰────────────────────────────────");
}

pub fn list_references(id: &str) {
    let home = dirs::home_dir().unwrap();

    let references_dir = home
        .join(".lattice")
        .join("references");

    let entries = match fs::read_dir(&references_dir) {
        Ok(entries) => entries,
        Err(_) => {
            println!("no references found");
            return;
        }
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

        let reference_id = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

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

        if from_document != id && to_document != id {
            continue;
        }

        references.push((
            reference_id,
            from_document,
            from_version,
            from_span,
            to_document,
            to_version,
            to_span,
        ));
    }

    references.sort_by(|a, b| {
        let a_num = a.0
            .trim_start_matches('R')
            .parse::<usize>()
            .unwrap_or(0);

        let b_num = b.0
            .trim_start_matches('R')
            .parse::<usize>()
            .unwrap_or(0);

        a_num.cmp(&b_num)
    });

    println!();
    println!("╭─ References for {}", id);
    println!("│");

    if references.is_empty() {
        println!("│  No references found.");
        println!("│");
        println!("╰────────────────────────────────────────");
        return;
    }

    println!(
        "│  {:<6} {:<18} {:<18}",
        "ID", "From", "To"
    );

    println!(
        "│  ────── ────────────────── ──────────────────"
    );

    for (
        reference_id,
        from_document,
        from_version,
        from_span,
        to_document,
        to_version,
        to_span,
    ) in references {
        println!(
            "│  {:<6} {:<18} {:<18}",
            reference_id,
            format!("{}:{}:{}", from_document, from_version, from_span),
            format!("{}:{}:{}", to_document, to_version, to_span),
        );
    }

    println!("│");
    println!("╰────────────────────────────────────────");
}

pub fn remove_reference(reference_id: &str) {
    let home = dirs::home_dir().unwrap();

    let reference_path = home
        .join(".lattice")
        .join("references")
        .join(reference_id);

    if !reference_path.is_file() {
        println!("reference not found: {}", reference_id);
        return;
    }

    match fs::remove_file(&reference_path) {
        Ok(_) => {
            println!("removed reference: {}", reference_id);
        }
        Err(_) => {
            println!("failed to remove reference: {}", reference_id);
        }
    }
}
