use std::env;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use tokio::stream;

fn handle_get_request(stream: &mut TcpStream, path: &str, folder: &PathBuf){
    let file_path = folder.join(&path[1..]);
    let raspuns = match fs::read_to_string(&file_path) {
        Ok(content) => {
            let content_type = determine_content_type(&file_path);
            format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nConnection: closed\r\n\r\n{}", content_type, content)
        }
        Err(_) => "HTTP/1.1 404 Not Found\r\nConnections: closed\r\n\r\n".to_string(),
    };
    stream.write(raspuns.as_bytes()).expect("Failed to write response");
    stream.flush().expect("Failed to  flush stream");
}

fn respond_with_status(stream: &mut TcpStream, status: &str) {
    let raspuns = format!("HTTP/1.1 {}\r\nConnection: closed\r\n\r\n", status);
    stream.write(raspuns.as_bytes()).expect("Failed to write response");
}

fn determine_content_type(file_path: &PathBuf) -> String {
    match file_path.extension() {
        Some(ext) => {
            match ext.to_str().unwrap(){
            "txt" => "text/plain; charset=utf-8".to_string(),
            "html" => "text/html; charset=utf-8".to_string(),
            "css" => "text/css; charset=utf-8".to_string(),
            "js" => "text/javascript; charset=utf-8".to_string(),
            "jpeg" | "jpg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "zip" => "application/zip".to_string(),
            _ => "application/octet-stream".to_string(),
          }
        }
        None => "application/octet-stream".to_string(),
    }
}

fn parse_header_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, ": ");
    match (parts.next(), parts.next()) {
        (Some(key), Some(value)) => Some((key, value)),
        _ => None,
    }
}

fn handle_connection(mut stream: TcpStream, folder: &PathBuf) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read request");
    let request = String::from_utf8_lossy(&buffer);
    let mut lines = request.lines();
    let req_line = lines.next().expect("No request line found");
    let mut parts = req_line.split_whitespace();
    let method = parts.next().expect("No method found");
    let path = parts.next().expect("No path found");
    match method {
        "GET" => handle_get_request(&mut stream, path, folder),
        "POST" => handle_post_request(&mut stream, path, folder, lines),
        _ => {
            respond_with_status(&mut stream, "405 Method Not Allowed");
        }
    }
}

fn handle_post_request(stream: &mut TcpStream, path: &str, folder: &PathBuf, lines: std::str::Lines) {
    let script = folder.join("scripts").join(&path[1..]);
    if script.exists() && script.is_file() {
        let mut command = Command::new(script);
        for line in lines {
            if let Some((key, value)) = parse_header_line(line){
                command.env(key, value);
            }
        }
        command.env("Method", "POST").env("Path", path);
        let output = command.stdout(Stdio::piped()).spawn().expect("Failed to execute script").wait_with_output().expect("Failed to wait for script execution");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status_coode = if output.status.success() {"200 OK"} else {"500 Internal Server Error"};
        let response = format!("HTTP/1.1 {}\r\nConnection: closed\r\n\r\n{}", status_coode, if output.status.success() {stdout} else {stderr});
        stream.write(response.as_bytes()).expect("failed to write response");
    } else {
        respond_with_status(stream, "404 Not Found");
    }
}

fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} PORT ROOT_FOLDER", args[0]);
        return;
    }
    let port = args[1].parse::<u16>().expect("Invalid port number");
    let folder = PathBuf::from(&args[2]);
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).expect("Failed to bind to port");
    println!("Root folder: {}", folder.display());
    println!("Server listening on 0.0.0.0:{}", port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clone = folder.clone();
                thread::spawn(move || {
                    handle_connection(stream, &clone);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

}