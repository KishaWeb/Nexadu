use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

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

        _ => "ERROR unknown request\n".to_string(),
    }
}

fn document_path(id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let path = home
        .join(".lattice")
        .join("documents")
        .join(id);

    if path.is_dir() {
        Some(path)
    } else {
        None
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

pub fn serve(address: &str) -> io::Result<()> {
    let server = Server::bind(address)?;
    server.run()
}
