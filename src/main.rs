use std::{env, fs, path::Path};

mod spans;
mod version_control;
mod tui;
mod references;
mod networking;

use version_control::{history, restore, status, version};

use spans::{create_span, show_span,remove_span,gc, list_spans};

use tui::{tui, tui_remote};

use references::{create_reference, show_reference,list_references,remove_reference};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || (args.len() < 3 && args[1] != "serve") {
        println!(
            "usage: Nexadu <init|view|version|history|status|restore|span-create|span|spans-list|span-remove|gc|ref-create|ref|ref-list|ref-remove|serve> <folder|id|address> [version|file|span|reference]"
        );
        return;
    }

    match args[1].as_str() {
        "init" => init(Path::new(&args[2])),

        
        "view" => {
            if args.len() < 3 {
                eprintln!(
                    "usage: Nexadu view <id> [version] [tui] [--remote ADDRESS]"
                );
                return;
            }

            let remote_index = args.iter().position(|arg| arg == "--remote");

            if let Some(index) = remote_index {
                let address = args
                    .get(index + 1)
                    .map(String::as_str)
                    .unwrap_or("127.0.0.1:9000");

                let before_remote = &args[3..index];
                let version = before_remote
                    .iter()
                    .find(|arg| arg.as_str() != "tui")
                    .map(String::as_str);

                if before_remote.iter().any(|arg| arg == "tui") {
                    if let Err(error) =
                        tui_remote(&args[2], version, address)
                    {
                        eprintln!("remote TUI error: {}", error);
                    }
                } else {
                    let request = match version {
                        Some(version) => {
                            format!("GET {} {}", args[2], version)
                        }
                        None => format!("GET {}", args[2]),
                    };

                    match networking::connect(
                        address,
                        &format!("{}\n", request),
                    ) {
                        Ok(response) => print!("{}", response),
                        Err(error) => {
                            eprintln!("connection failed: {}", error);
                        }
                    }
                }
            } else if args.len() >= 4 && args[3] == "tui" {
                if let Err(error) = tui(&args[2], None) {
                    eprintln!("TUI error: {}", error);
                }
            } else if args.len() >= 5 && args[4] == "tui" {
                if let Err(error) = tui(&args[2], Some(&args[3])) {
                    eprintln!("TUI error: {}", error);
                }
            } else if args.len() == 3 {
                view(&args[2], None);
            } else {
                view(&args[2], Some(&args[3]));
            }
        }

        "client" => {
            if args.len() < 4 {
                println!("usage: Nexadu client <address> <request>");
                return;
            }
            let request = args[3..].join(" ");
            if let Err(error) = send_request(&args[2], &request) {
                eprintln!("client error: {}", error);
            }
        }

        "version" => version(&args[2]),
        "history" => history(&args[2]),
        "status" => status(&args[2]),

        "restore" => {
            if args.len() < 4 {
                println!("usage: Nexadu restore <id> <version>");
                return;
            }

            restore(&args[2], &args[3]);
        }

        "span-create" => {
            if args.len() < 7 {
                println!(
                    "usage: Nexadu span-create <id> <version|current> <file> <start> <end>"
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
                println!("usage: Nexadu span <id> <version> <span> [all]");
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
                println!("usage: Nexadu list_spans <id>")
            }else if args.len() == 3{
                list_spans(&args[2], None)
            }else{
                list_spans(&args[2], Some(&args[3]));
            }
        }

        "span-remove" => {
            if args.len() < 4 {
                println!("usage: Nexadu span-remove <id> <span>");
                return;
            }

            remove_span(&args[2], &args[3]);
        }

        "gc" => {
            if args.len() < 3 {
                println!("usage: Nexadu gc <id>")
            }

            gc(&args[2]);
        }

        "ref-create" => {
            if args.len() < 8 {
                println!(
                    "usage: Nexadu ref-create <from_document> <from_version> <from_span> <to_document> <to_version> <to_span>"
                );
                return;
            }

            create_reference(
                &args[2],
                &args[3],
                &args[4],
                &args[5],
                &args[6],
                &args[7],
            );
        }
        
        "ref" => {
            if args.len() < 4 {
                println!("usage: Nexadu ref <id> <reference>");
                return;
            }

            show_reference(&args[2], &args[3]);
        }
        
        "ref-list" => {
            if args.len() < 3 {
                println!("usage: Nexadu ref-list <id>");
                return;
            }

            list_references(&args[2]);
        }
        "ref-remove" => {
            if args.len() < 3 {
                println!("usage: Nexadu ref-remove <reference>");
                return;
            }

            remove_reference(&args[2]);
        }

"serve" => {
        let address = if args.len() >= 3 {
                &args[2]
            } else {
                "127.0.0.1:9000"
            };

            if let Err(error) = networking::serve(address) {
                eprintln!("server error: {}", error);
            }
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
    let documents = home.join(".Nexadu").join("documents");

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

    let document_dir = home.join(".Nexadu").join("documents").join(id);

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
        println!("version not found: {}", version.unwrap_or("current"));
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

fn send_request(address: &str, request: &str) -> std::io::Result<()> {
    let response = networking::connect(address, &format!("{}\n", request))?;
    print!("{}", response);
    Ok(())
}
