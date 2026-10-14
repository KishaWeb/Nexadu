use std::{env, fs, io::{self, Read, Write}, net::{TcpListener, TcpStream}, path::{Path, PathBuf}};

mod spans;
mod version_control;
mod tui;
mod references;

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
            if args.len() == 3 {
                view(&args[2], None);
            } else if args.len() >= 5 && args[3] == "tui" && args[4] == "--remote" {
                let address = args.get(5).map(String::as_str).unwrap_or("127.0.0.1:9000");
                if let Err(error) = tui_remote(&args[2], None, address) {
                    eprintln!("remote TUI error: {}", error);
                }
            } else if args.len() >= 6 && args[4] == "tui" && args[5] == "--remote" {
                let address = args.get(6).map(String::as_str).unwrap_or("127.0.0.1:9000");
                if let Err(error) = tui_remote(&args[2], Some(&args[3]), address) {
                    eprintln!("remote TUI error: {}", error);
                }
            } else if args.len() >= 4 && args[3] == "tui" {
                let _ = tui(&args[2], None);
            } else if args.len() >= 5 && args[4] == "tui" {
                let _ = tui(&args[2], Some(&args[3]));
            } else if args.len() >= 4 && args[3] == "--remote" {
                let address = args.get(4).map(String::as_str).unwrap_or("127.0.0.1:9000");
                let request = format!("GET {}", args[2]);
                if let Err(error) = send_request(address, &request) {
                    eprintln!("client error: {}", error);
                }
            } else if args.len() >= 5 && args[4] == "--remote" {
                let address = args.get(5).map(String::as_str).unwrap_or("127.0.0.1:9000");
                let request = format!("GET {} {}", args[2], args[3]);
                if let Err(error) = send_request(address, &request) {
                    eprintln!("client error: {}", error);
                }
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

            if let Err(error) = serve(address) {
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

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn bind(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;

        println!("Nexadu server listening on {}", address);

        Ok(Self { listener })
    }

    pub fn run(&self) -> io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(error) = handle_client(stream) {
                        eprintln!("client error: {}", error);
                    }
                }

                Err(error) => {
                    eprintln!("connection failed: {}", error);
                }
            }
        }

        Ok(())
    }
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0u8; 8192];

    let bytes = stream.read(&mut buffer)?;

    if bytes == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes]);

    let response = handle_request(request.trim());

    stream.write_all(response.as_bytes())?;

    Ok(())
}

fn handle_request(request: &str) -> String {
    let args: Vec<&str> = request.split_whitespace().collect();

    if args.is_empty() {
        return "ERROR empty request\n".to_string();
    }

    match args[0] {
        "PING" => "PONG\n".to_string(),

        "INFO" => {
            "NEXADU\nVERSION 1\nPROTOCOL 1\n".to_string()
        }

        "GET" => match args.len() {
            2 => view_document(args[1], None),
            3 => view_document(args[1], Some(args[2])),
            4 => view_file(args[1], args[2], args[3]),
            _ => "ERROR invalid GET syntax\n".to_string(),
        },

        "LIST" => match args.len() {
            2 => list_remote_files(args[1], "current"),
            3 => list_remote_files(args[1], args[2]),
            _ => "ERROR invalid LIST syntax\n".to_string(),
        },

        "READ" => match args.len() {
            4 => read_remote_file(args[1], args[2], args[3]),
            _ => "ERROR invalid READ syntax\n".to_string(),
        },

        _ => "ERROR unknown request\n".to_string(),
    }
}

fn document_path(id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let path = home
        .join(".Nexadu")
        .join("documents")
        .join(id);

    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}


fn resolve_view_dir(document_dir: &Path, version: &str) -> Option<PathBuf> {
    if version == "current" {
        let metadata = fs::read_to_string(document_dir.join("metadata")).ok()?;
        let path = metadata
            .lines()
            .find_map(|line| line.strip_prefix("path="))?;
        let path = PathBuf::from(path);
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    } else {
        let path = document_dir.join("versions").join(version);
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    }
}

fn list_remote_files(id: &str, version: &str) -> String {
    let Some(document_dir) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };

    let Some(view_dir) = resolve_view_dir(&document_dir, version) else {
        return format!("ERROR version not found: {}\n", version);
    };

    let mut files = match fs::read_dir(view_dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() {
                    path.file_name().map(|name| name.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(error) => return format!("ERROR could not list files: {}\n", error),
    };

    files.sort();
    if files.is_empty() {
        return String::new();
    }
    format!("{}\n", files.join("\n"))
}

fn read_remote_file(id: &str, version: &str, file: &str) -> String {
    let requested_path = Path::new(file);
    if requested_path.file_name().and_then(|name| name.to_str()) != Some(file)
        || requested_path.components().count() != 1
    {
        return "ERROR invalid filename\n".to_string();
    }

    let Some(document_dir) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };

    let Some(view_dir) = resolve_view_dir(&document_dir, version) else {
        return format!("ERROR version not found: {}\n", version);
    };

    let file_path = view_dir.join(file);
    if !file_path.is_file() {
        return format!("ERROR file not found: {}\n", file);
    }

    match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(error) => format!("ERROR could not read file: {}\n", error),
    }
}

fn view_document(id: &str, version: Option<&str>) -> String {
    let Some(document_dir) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };

    let view_dir = match version {
        Some(version) => document_dir.join("versions").join(version),

        None => {
            let metadata = match fs::read_to_string(document_dir.join("metadata")) {
                Ok(metadata) => metadata,
                Err(_) => return "ERROR metadata not found\n".to_string(),
            };

            let Some(path) = metadata
                .lines()
                .find_map(|line| line.strip_prefix("path="))
            else {
                return "ERROR document path not found\n".to_string();
            };

            PathBuf::from(path)
        }
    };

    if !view_dir.is_dir() {
        return format!("ERROR version not found: {}\n", version.unwrap_or("current"));
    }

    let mut output = String::new();

    output.push_str(&format!("Document: {}\n", id));

    match version {
        Some(version) => output.push_str(&format!("Version: {}\n", version)),
        None => output.push_str("Version: current\n"),
    }

    output.push('\n');

    append_directory(&view_dir, "", &mut output);

    output
}

fn view_file(id: &str, version: &str, file: &str) -> String {
    let Some(document_dir) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };

    let file_path = document_dir
        .join("versions")
        .join(version)
        .join(file);

    if !file_path.is_file() {
        return format!("ERROR file not found: {}\n", file);
    }

    match fs::read_to_string(&file_path) {
        Ok(contents) => {
            format!(
                "Document: {}\nVersion: {}\nFile: {}\n\n{}",
                id, version, file, contents
            )
        }

        Err(error) => {
            format!("ERROR could not read file: {}\n", error)
        }
    }
}

fn append_directory(
    directory: &Path,
    indent: &str,
    output: &mut String,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            output.push_str(&format!("{}{}/\n", indent, name));

            append_directory(
                &path,
                &format!("{}  ", indent),
                output,
            );
        } else {
            output.push_str(&format!("{}{}\n", indent, name));

            if let Ok(contents) = fs::read_to_string(&path) {
                for line in contents.lines() {
                    output.push_str(&format!("{}  {}\n", indent, line));
                }
            }

            output.push('\n');
        }
    }
}

fn send_request(address: &str, request: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(format!("{}\n", request).as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    print!("{}", response);
    Ok(())
}

pub fn serve(address: &str) -> io::Result<()> {
    let server = Server::bind(address)?;
    server.run()
}
