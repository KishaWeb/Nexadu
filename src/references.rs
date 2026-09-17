 use std::fs;

pub fn create_reference(
    from_document: &str,
    from_span: &str,
    to_document: &str,
    to_span: &str,
) {
    let home = dirs::home_dir().unwrap();
    let lattice_dir = home.join(".lattice");
    let references_dir = lattice_dir.join("references");

    fs::create_dir_all(&references_dir).unwrap();

    // Make sure both documents exist
    for document in [from_document, to_document] {
        let document_dir = lattice_dir.join("documents").join(document);

        if !document_dir.is_dir() {
            println!("document not found: {}", document);
            return;
        }
    }

    // Make sure both spans exist
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
        "from_document={}\nfrom_span={}\nto_document={}\nto_span={}\n",
        from_document,
        from_span,
        to_document,
        to_span,
    );

    fs::write(&reference_path, metadata).unwrap();

    println!(
        "{}: {}:{} → {}:{}",
        reference_id,
        from_document,
        from_span,
        to_document,
        to_span
    );
}
