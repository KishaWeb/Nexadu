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
    let versions_dir = document_dir.join("versions");

    fs::create_dir_all(&versions_dir).unwrap();

    let path = fs::canonicalize(folder).unwrap();

    fs::write(
        document_dir.join("metadata"),
        format!(
            "id={}\npath={}\nversion=0",
            id,
            path.to_string_lossy()
        ),
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

    let versions_dir = document_dir.join("versions");

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let metadata = fs::read_to_string(document_dir.join("metadata")).unwrap();

    let path = metadata
        .lines()
        .find_map(|line| line.strip_prefix("path="))
        .map(Path::new);

    let path = match path {
        Some(path) if path.is_dir() => path,
        _ => {
            println!("document path not found");
            return;
        }
    };

    let version_number = fs::read_dir(&versions_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count();

    if version_number > 0 {
        let latest_version = versions_dir.join(format!("V{}", version_number));

        if directories_equal(path, &latest_version) {
            println!("nothing to version");
            return;
        }
    }

    let new_version_number = version_number + 1;
    let version_dir = versions_dir.join(format!("V{}", new_version_number));

    fs::create_dir_all(&version_dir).unwrap();

    copy_directory(path, &version_dir);

    let mut new_metadata = String::new();

    for line in metadata.lines() {
        if line.starts_with("version=") {
            continue;
        }

        new_metadata.push_str(line);
        new_metadata.push('\n');
    }

    new_metadata.push_str(&format!("version={}", new_version_number));

    fs::write(document_dir.join("metadata"), new_metadata).unwrap();

    println!("created {} V{}", id, new_version_number);
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
                        return;
                    }
                }
            }
            Err(_) => {
                println!("metadata not found");
                return;
            }
        },
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

fn directories_equal(current: &Path, version: &Path) -> bool {
    let current_entries: Vec<_> = fs::read_dir(current)
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    let version_entries: Vec<_> = fs::read_dir(version)
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    if current_entries.len() != version_entries.len() {
        return false;
    }

    for current_entry in current_entries {
        let current_path = current_entry.path();
        let name = current_entry.file_name();
        let version_path = version.join(&name);

        if current_path.is_dir(){
            if !version_path.is_dir(){
                return false;
            }

            if directories_equal(&current_path, &version_path){
                return false;
            }
        }else {
            if !version_path.is_path(){
                return false;
            }

            let current_contenets = fs::read(&current_path);
            let version_contenets = fs::read(&version_path);

            if current_contenets != version_contenets{
                return false;
            }
        }
    }

    true
}
