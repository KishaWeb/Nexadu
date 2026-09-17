use std::{env, fs, path::Path};

mod spans;
mod version_control;
mod tui;
mod references;

use version_control::{history, restore, status, version};

use spans::{create_span, show_span,remove_span,gc, list_spans};

use tui::tui;

use references::create_reference;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(
            "usage: lattice <init|view|version|history|status|restore|span-create|span|spans-list|span-remove|gc> <folder|id> [version|file|span]"
        );
        return;
    }

    match args[1].as_str() {
        "init" => init(Path::new(&args[2])),

        "view" => {
            if args.len() == 3 {
                view(&args[2], None);
            } else if args.len() >= 4 && args[3] == "tui" {
                let _ = tui(&args[2], None);
            } else if args.len() >= 5 && args[4] == "tui" {
                let _ = tui(&args[2], Some(&args[3]));
            } else {
                view(&args[2], Some(&args[3]));
            }
        }

        "version" => version(&args[2]),
        "history" => history(&args[2]),
        "status" => status(&args[2]),

        "restore" => {
            if args.len() < 4 {
                println!("usage: lattice restore <id> <version>");
                return;
            }

            restore(&args[2], &args[3]);
        }

        "span-create" => {
            if args.len() < 7 {
                println!(
                    "usage: lattice span-create <id> <version|current> <file> <start> <end>"
                );
                return;
            }

            let start: usize = match args[5].parse() {
                Ok(value) => value,
                Err(_) => {
                    println!("invalid start");
                    return;
                }
            };

            let end: usize = match args[6].parse() {
                Ok(value) => value,
                Err(_) => {
                    println!("invalid end");
                    return;
                }
            };

            create_span(
                &args[2],
                &args[3],
                &args[4],
                start,
                end,
            );
        }
        
        "span" => {
            if args.len() < 5 {
                println!("usage: lattice span <id> <version> <span> [all]");
                return;
            }

            let all = args.last().map(|arg| arg == "all").unwrap_or(false);

            show_span(
                &args[2],
                &args[3],
                &args[4],
                all,
            );
        }
        
        "spans-list" => {
            if args.len() < 3{
                println!("usage: lattice list_spans <id>")
            }else if args.len() == 3{
                list_spans(&args[2], None)
            }else{
                list_spans(&args[2], Some(&args[3]));
            }
        }

        "span-remove" => {
            if args.len() < 4 {
                println!("usage: lattice span-remove <id> <span>");
                return;
            }

            remove_span(&args[2], &args[3]);
        }

        "gc" => {
            if args.len() < 3 {
                println!("usage: lattice gc <id>")
            }

            gc(&args[2]);
        }

        "ref-create" => {
            if args.len() < 6 {
                println!("usage: lattice ref-create <from_document> <from_span> <to_document> <to_span>");
                return;
            }

            create_reference(
                &args[2],
                &args[3],
                &args[4],
                &args[5],
            );
        }

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
        format!("id={}\npath={}\nversion=0", id, path.to_string_lossy()),
    )
    .unwrap();

    println!("registered {}", id);
}

fn view(id: &str, version: Option<&str>) {
    let home = dirs::home_dir().unwrap();

    let document_dir = home.join(".lattice").join("documents").join(id);

    if !document_dir.is_dir() {
        println!("document not found: {}", id);
        return;
    }

    let view_dir = match version {
        Some(version) => document_dir.join("versions").join(version),

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
