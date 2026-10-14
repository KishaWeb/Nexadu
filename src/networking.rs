use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
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
                Ok(mut stream) => {
                    if let Err(error) = handle_client(&mut stream) {
                        eprintln!("client error: {}", error);
                    }
                }
                Err(error) => eprintln!("connection failed: {}", error),
            }
        }
        Ok(())
    }
}

fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
    let mut request = String::new();
    stream.read_to_string(&mut request)?;
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
        "INFO" => "NEXADU\nVERSION 1\nPROTOCOL 1\n".to_string(),
        "GET" => match args.len() {
            2 => view_document(args[1], None),
            3 => view_document(args[1], Some(args[2])),
            4 => read_file(args[1], args[2], args[3]),
            _ => "ERROR invalid GET syntax\n".to_string(),
        },
        "LIST" => {
            if args.len() < 2 || args.len() > 3 {
                return "ERROR usage: LIST <id> [version]\n".to_string();
            }
            let version = args.get(2).copied().unwrap_or("current");
            list_files(args[1], version)
        }
        "READ" => {
            if args.len() != 4 {
                return "ERROR usage: READ <id> <version> <relative-file-path>\n".to_string();
            }
            read_file(args[1], args[2], args[3])
        }
        _ => format!("ERROR unknown request: {}\n", args[0]),
    }
}

fn nexadu_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".Nexadu")
}

fn document_path(id: &str) -> Option<PathBuf> {
    if !is_single_component(id) {
        return None;
    }
    let path = nexadu_path().join("documents").join(id);
    path.is_dir().then_some(path)
}

fn is_single_component(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().count() == 1
        && Path::new(value).components().all(|part| matches!(part, Component::Normal(_)))
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().all(|part| matches!(part, Component::Normal(_)))
}

fn view_directory(document: &Path, version: &str) -> io::Result<PathBuf> {
    if version == "current" {
        let metadata = fs::read_to_string(document.join("metadata"))?;
        let source = metadata
            .lines()
            .find_map(|line| line.strip_prefix("path="))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "metadata has no path"))?;
        let path = PathBuf::from(source);
        if !path.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "current document directory not found"));
        }
        Ok(path)
    } else {
        if !is_single_component(version) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid version"));
        }
        let path = document.join("versions").join(version);
        if path.is_dir() { Ok(path) } else {
            Err(io::Error::new(io::ErrorKind::NotFound, format!("version not found: {}", version)))
        }
    }
}

fn view_document(id: &str, version: Option<&str>) -> String {
    let Some(document) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };
    let version_name = version.unwrap_or("current");
    let root = match view_directory(&document, version_name) {
        Ok(path) => path,
        Err(error) => return format!("ERROR {}\n", error),
    };

    let mut output = format!("Document: {}\nVersion: {}\n\n", id, version_name);
    if let Err(error) = append_directory(&root, &root, "", &mut output) {
        return format!("ERROR {}\n", error);
    }
    output
}

fn append_directory(root: &Path, directory: &Path, indent: &str, output: &mut String) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            output.push_str(&format!("{}{}/\n", indent, name));
            append_directory(root, &path, &format!("{}  ", indent), output)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            output.push_str(&format!("{}{}\n", indent, relative));
            if let Ok(contents) = fs::read_to_string(&path) {
                for line in contents.lines() {
                    output.push_str(&format!("{}  {}\n", indent, line));
                }
            }
            output.push('\n');
        }
    }
    Ok(())
}

fn list_files(id: &str, version: &str) -> String {
    let Some(document) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };
    let root = match view_directory(&document, version) {
        Ok(path) => path,
        Err(error) => return format!("ERROR {}\n", error),
    };

    fn collect(root: &Path, directory: &Path, files: &mut Vec<String>) -> io::Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect(root, &path, files)?;
            } else if path.is_file() {
                let relative = path.strip_prefix(root)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    if let Err(error) = collect(&root, &root, &mut files) {
        return format!("ERROR {}\n", error);
    }
    files.sort();
    if files.is_empty() { String::new() } else { format!("{}\n", files.join("\n")) }
}

fn read_file(id: &str, version: &str, file: &str) -> String {
    if !safe_relative_path(file) {
        return "ERROR invalid file path\n".to_string();
    }
    let Some(document) = document_path(id) else {
        return format!("ERROR document not found: {}\n", id);
    };
    let root = match view_directory(&document, version) {
        Ok(path) => path,
        Err(error) => return format!("ERROR {}\n", error),
    };
    let path = root.join(file);
    if !path.is_file() {
        return format!("ERROR file not found: {}\n", file);
    }
    match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => format!("ERROR could not read file: {}\n", error),
    }
}

pub fn serve(address: &str) -> io::Result<()> {
    Server::bind(address)?.run()
}

pub fn connect(address: &str, request: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect(address)?;
    stream.write_all(request.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}
