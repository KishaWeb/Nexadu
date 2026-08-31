use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("usage: lattice <init|view> <folder|id>");
        return;
    }

    match args[1].as_str() {
        "init" => init(Path::new(&args[2])),
        "view" => view(&args[2]),
        _ => println!("unknown command: {}", args[1]),
    }
}

fn init(folder: &Path) {
    if !folder.is_dir() {
        println!("document must be a directory");
        return;
    }

    let home = dirs::home_dir().unwrap();
    let documents = home.join(".lattice").join("documents");

    fs::create_dir_all(&documents).unwrap();

    let id = format!("D{}", fs::read_dir(&documents).unwrap().count() + 1);
    let document_dir = documents.join(&id);
    let current_dir = document_dir.join("current");
    let versions_dir = document_dir.join("versions");

    fs::create_dir_all(&current_dir).unwrap();
    fs::create_dir_all(&versions_dir).unwrap();

    copy_directory(folder, &current_dir);

    let entry = fs::read_dir(&current_dir)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .unwrap_or_default();

    fs::write(
        document_dir.join("metadata"),
        format!("id={}\nentry={}", id, entry),
    )
    .unwrap();

    println!("registered {}", id);
}

fn copy_directory(source: &Path, destination: &Path) {
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            fs::create_dir_all(&destination_path).unwrap();
            copy_directory(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).unwrap();
        }
    }
}

fn view(id: &str) {
    let home = dirs::home_dir().unwrap();
    let document_dir = home.join(".lattice").join("documents").join(id);
    let current_dir = document_dir.join("current");

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    println!("Document: {}", id);
    println!();

    print_directory(&current_dir, "");
}

fn print_directory(directory: &Path, indent: &str) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            println!("{}{}/", indent, name);
            print_directory(&path, &format!("{}  ", indent));
        } else {
            println!("{}{}", indent, name);

            if let Ok(contents) = fs::read_to_string(&path) {
                for line in contents.lines() {
                    println!("{}  {}", indent, line);
                }
            }

            println!();
        }
    }
}
