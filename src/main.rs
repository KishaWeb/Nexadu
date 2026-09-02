use std::{
    env,
    fs,
    path::Path,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("usage: lattice <init|view|version|history> <folder|id>");
        return;
    }

    match args[1].as_str() {
        "init" => init(Path::new(&args[2])),
        "view" => {
            if args.len() == 3 {
                view(&args[2], None);
            } else {
                view(&args[2], Some(&args[3]));
            }
        }
        "version" => version(&args[2]),
        "history" => history(&args[2]),
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

fn version(id: &str) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    let current_dir = document_dir.join("current");
    let versions_dir = document_dir.join("versions");

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let version_number = fs::read_dir(&versions_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count()
        + 1;

    let version_dir = versions_dir.join(format!("V{}", version_number));

    fs::create_dir_all(&version_dir).unwrap();

    copy_directory(&current_dir, &version_dir);

    println!("created {} V{}", id, version_number);

    let metadata = fs::read_to_string(document_dir.join("metadata")).unwrap();

    let mut new_metadata = String::new();

    for line in metadata.lines() {
        if line.starts_with("version=") {
            continue;
        }

        new_metadata.push_str(line);
        new_metadata.push('\n');
    }

    new_metadata.push_str(&format!("version={}", version_number));

    fs::write(document_dir.join("metadata"), new_metadata).unwrap();
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

fn view(id: &str, version: Option<&str>) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let view_dir = match version {
        Some(version) => document_dir
            .join("versions")
            .join(version),

        None => document_dir.join("current"),
    };

    if !view_dir.is_dir() {
        println!("version not found: {}", version.unwrap());
        return;
    }

    println!("Document: {}", id);

    if let Some(version) = version {
        println!("Version: {}", version);
    } else {
        println!("Current");
    }

    println!();

    print_directory(&view_dir, "");
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

fn history(id: &str) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home
        .join(".lattice")
        .join("documents")
        .join(id);

    let versions_dir = document_dir.join("versions");

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let mut versions = Vec::new();

    for entry in fs::read_dir(&versions_dir).unwrap() {
        let entry = entry.unwrap();

        if entry.path().is_dir() {
            versions.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    versions.sort();

    for version in versions {
        println!("{}", version);
    }

    let metadata = fs::read_to_string(document_dir.join("metadata")).unwrap();

    for line in metadata.lines() {
        if let Some(version) = line.strip_prefix("version=") {
            println!("Current version: V{}", version);
        }
    }
}
