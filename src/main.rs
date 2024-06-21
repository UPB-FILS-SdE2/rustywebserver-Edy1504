/*use std::env;
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

}*/
mod runner;

use std::env;
use std::error::Error;
use std::ffi::c_int;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use std::os::raw::c_void;

use nix::libc::{Elf32_Ehdr, Elf32_Phdr, siginfo_t, AT_ENTRY, AT_PHDR};
use nix::sys::mman::{mmap, munmap, mprotect, MapFlags, ProtFlags};
use nix::sys::signal::{sigaction, SigAction, SigHandler, SaFlags, SigSet, Signal};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::num::NonZeroUsize;

lazy_static! {
    static ref SEGMENTS: Mutex<HashMap<usize, (usize, usize, u32)>> = Mutex::new(HashMap::new());
}

fn read_elf_header(file: &mut File) -> Result<Elf32_Ehdr, Box<dyn Error>> {
    let mut ehdr: Elf32_Ehdr = unsafe { std::mem::zeroed() };
    file.seek(SeekFrom::Start(0))?;
    let ehdr_slice = unsafe { std::slice::from_raw_parts_mut(&mut ehdr as *mut _ as *mut u8, mem::size_of::<Elf32_Ehdr>()) };
    file.read_exact(ehdr_slice)?;
    Ok(ehdr)
}

fn read_program_headers(file: &mut File, ehdr: &Elf32_Ehdr) -> Result<Vec<Elf32_Phdr>, Box<dyn Error>> {
    let mut phdrs = vec![unsafe { std::mem::zeroed() }; ehdr.e_phnum as usize];
    file.seek(SeekFrom::Start(ehdr.e_phoff as u64))?;
    let phdr_slice = unsafe { std::slice::from_raw_parts_mut(phdrs.as_mut_ptr() as *mut u8, (ehdr.e_phnum as usize * mem::size_of::<Elf32_Phdr>())) };
    file.read_exact(phdr_slice)?;
    Ok(phdrs)
}

fn display_segments(phdrs: &[Elf32_Phdr]) {
    eprintln!("Segments");
    eprintln!("#\taddress\t\tsize\toffset\tlength\tflags");
    for (i, phdr) in phdrs.iter().enumerate() {
        let flags = format!(
            "{}{}{}",
            if phdr.p_flags & 0x1 != 0 { "x" } else { "-" },
            if phdr.p_flags & 0x2 != 0 { "w" } else { "-" },
            if phdr.p_flags & 0x4 != 0 { "r" } else { "-" },
        );
        eprintln!(
            "{}\t0x{:x}\t{}\t0x{:x}\t{}\t{}",
            i, phdr.p_vaddr, phdr.p_memsz, phdr.p_offset, phdr.p_filesz, flags
        );
    }
}

unsafe extern "C" fn sigsegv_handler(_signal: c_int, siginfo: *mut siginfo_t, _extra: *mut c_void) {
    let faulting_address = (*siginfo).si_addr() as usize;
    let segments = SEGMENTS.lock().unwrap();

    for (start, &(end, offset, flags)) in segments.iter() {
        if faulting_address >= *start && faulting_address < *start + end {
            let page_size = 4096; // Assuming page size of 4096 bytes
            let page_start = faulting_address & !(page_size - 1);

            if let Ok(_) = mmap(
                NonZeroUsize::new(page_start as usize),
                NonZeroUsize::new(page_size).expect("REASON"),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_PRIVATE | MapFlags::MAP_FIXED,
                offset as c_int,
                page_start as i64 - *start as i64,
            ) {
                mprotect(
                    page_start as *mut c_void,
                    page_size,
                    ProtFlags::from_bits(flags.try_into().unwrap()).unwrap(),
                )
                .unwrap(); // You should handle errors more gracefully
                return;
            }
        }
    }

    eprintln!("Error: Unauthorized memory access at {:x}", faulting_address);
    std::process::exit(-200);
}

fn exec(filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(filename)?;
    let ehdr = read_elf_header(&mut file)?;
    let phdrs = read_program_headers(&mut file, &ehdr)?;

    display_segments(&phdrs);

    let base_address = phdrs.iter().map(|phdr| phdr.p_vaddr).min().unwrap_or(0);
    eprintln!("Base address 0x{:x}", base_address);
    eprintln!("Entry point 0x{:x}", ehdr.e_entry);

    let mut segments = HashMap::new();
    for phdr in &phdrs {
        segments.insert(
            phdr.p_vaddr as usize,
            (phdr.p_memsz as usize, phdr.p_offset as usize, phdr.p_flags),
        );
    }
    SEGMENTS.lock().unwrap().extend(segments);

    let env_address = 0; // Replace with actual logic to get env address

    runner::exec_run(base_address as usize, ehdr.e_entry as usize, env_address as usize);


    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <executable>", args[0]);
        std::process::exit(1);
    }
    exec(&args[1])
}
