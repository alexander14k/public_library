//! # Filename: main.rs
//!
//! Author: alexandre14k
//! Created: 2025-08-05
//!
//! License: MIT
//!
//! Object: Simple implementation of 
//!             RFC 2229 "A Dictionary Server Protocol"
//!             Published: October 1997
//!             https://www.rfc-editor.org/rfc/rfc2229.html
//!         with a csv dataset vocabulary based on topic : furniture.
//!
//! It includes both client and server modules.
//! 
//! Server specific :
//!     listen address
//!         ::1 (loopback)
//!     listen port
//!         tcp 2628

fn main() {
    if server::is_required() {
        let instance = server::Instance::new();
        instance.run()
    } else {
        let instance = client::Instance::new();
        instance.run();
    }
}

mod types {
    pub use std::env::args as env_args;
    pub use std::thread::spawn as thread_spawn;
    
    pub use std::io::BufReader as buffer_reader;
    pub use std::io::ErrorKind as error_kind;
    pub use std::io::stdin as console_input;
    
    pub use std::net::TcpListener as tcp_listener;
    pub use std::net::TcpStream as tcp_stream;
    pub use std::sync::Arc as atomic_ref;
    
    pub use std::time::Duration as time_duration;
    pub use std::process::exit as process_exit;
}

mod server {
    use crate::types;
    use std::io::{BufRead, Write};
    
    pub fn is_required() -> bool {
        types::env_args().any(|arg| arg == "-s")
    }
    
    pub struct Instance {
        dataset: Vec<String>,
    }

    impl Instance {
        pub fn new() -> Self {
            let raw_data = include_str!("dataset.csv").to_string();
            let vec_data = Self::check(raw_data);
            Self { dataset : vec_data }
        }

        pub fn run(&self) {
            let listener = self.start_listener("::1:2628");
            println!("Lexicon DICT server listening on ::1:2628");
            println!("End server by using Ctrl+C");
            self.handle_connections(listener);
        }

        fn start_listener(&self, addr: &str) -> types::tcp_listener {
            match types::tcp_listener::bind(addr) {
                Ok(listener) => {listener},
                Err(e) => {
                    eprintln!("Error bind {}: {}", addr, e);
                    types::process_exit(1);
                }
            }
        }

        fn handle_connections(&self, listener: types::tcp_listener) {
            let dataset = types::atomic_ref::new(self.dataset.clone());
            for stream in listener.incoming() {
                let dataset = dataset.clone();
                match stream {
                    Ok(stream) => {
                        types::thread_spawn(move || {
                            Self::handle_client(stream, dataset);
                        });
                    }
                    Err(e) => eprintln!("Error connect: {}", e),
                }
            }
        }

        fn handle_client(
                mut stream: types::tcp_stream, 
                dataset: types::atomic_ref<Vec<String>>) {
            let stream_clone = stream.try_clone().unwrap();
            let mut reader = types::buffer_reader::new(stream_clone);
            let _ = stream.write_all(b"220 DICT server ready\r\n");

            let mut line = String::new();
            while let Ok(bytes_read) = reader.read_line(&mut line) {
                if bytes_read == 0 {
                    break; // client closed connection
                }

                let trimmed = line.trim();
                if Self::handle_command(
                        trimmed, 
                        &mut stream, &dataset) {
                    break;
                }

                line.clear();
            }
        }

        fn handle_command(
                line: &str, 
                stream: &mut types::tcp_stream, 
                dataset: &Vec<String>) -> bool {
            match line.to_uppercase().as_str() {
                "QUIT" => {
                    let _ = stream.write_all(b"221 Bye\r\n");
                    true
                }
                _ if line.to_uppercase().starts_with("DEFINE") => {
                    Self::process_define(line, stream, dataset);
                    false
                }
                _ => {
                    let _ = stream.write_all(b"500 Unknown command\r\n");
                    false
                }
            }
        }

        fn process_define(
                line: &str, 
                stream: &mut types::tcp_stream, 
                dataset: &Vec<String>) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let word = parts[2];
                let response = Self::get(word, dataset.clone());

                if response.contains("not found") {
                    let _ = stream.write_all(b"552 No match\r\n");
                } else {
                    let _ = writeln!(stream, 
                        "150 Definitions found for \"{}\"\r\n{}", 
                            word, response);
                    let _ = stream.write_all(b"250 ok\r\n");
                }
            } else {
                let _ = stream.write_all(b"501 Syntax error in parameters\r\n");
            }
        }
        
        pub fn check(input_str : String) -> Vec<String> {
            let mut lines = input_str.lines();

            let header = match lines.next() {
                Some(value) => value,
                None => return vec![], 
                // if empty then check failed
            };

            let header_count = header.split(',').count();

            for line in lines.clone() {
                if line.split(',').count() != header_count {
                    return vec![]; 
                    // if not homogeneous then check failed 
                }
            }

            input_str.lines().map(|line| line.to_string()).collect()
        }
        
        fn get(word : &str, dict : Vec<String>) -> String {
            if dict.is_empty() {
                return "Empty dataset".to_string();
            }

            let header = &dict[0];
            let headers: Vec<&str> = header.split(',').collect();

            for line in dict.iter().skip(1) {
                let cols: Vec<&str> = line.split(',').collect();
                if cols.get(0).map_or(false, 
                        |&eng| eng.eq_ignore_ascii_case(word)) {
                    let mut result = String::new();
                    for (i, &col_name) in headers.iter().enumerate() {
                        let val = cols.get(i).unwrap_or(&"");
                        let output = format!("{} : {}\n", 
                            col_name.trim(), val.trim());
                        result.push_str(&output);
                    }
                    return result;
                }
            }

            "Missing word".to_string()
        }
    }
}

mod client {
    use crate::types;
    use std::io::{BufRead, Write};

    pub struct Instance {
        server_socket: String,
    }

    impl Instance {
        pub fn new() -> Self {
            Self {
                server_socket: "::1:2628".to_string(),
            }
        }

        pub fn run(&self) {
            loop {
                let word = self.read_user_input();
                if word.to_lowercase() == "!exit" {
                    println!("Good day!");
                    break;
                }

                match self.connect() {
                    Some(mut stream) => {
                        self.set_timeout(&mut stream);
                        self.send_request(&mut stream, &word);
                        self.read_response(&mut stream);
                    }
                    None => {
                        println!("Error connect to server.");
                        continue;
                    }
                }
            }
        }

        fn read_user_input(&self) -> String {
            println!("Translate word (or type '!exit' to exit): ");

            let mut input = String::new();
            match types::console_input().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => {
                    println!("Error input.");
                    String::new()
                }
            }
        }

        fn connect(&self) -> Option<types::tcp_stream> {
            match types::tcp_stream::connect(&self.server_socket) {
                Ok(stream) => Some(stream),
                Err(err) => {
                    println!("Connection error: {}", err);
                    None
                }
            }
        }

        fn set_timeout(&self, stream: &mut types::tcp_stream) {
            let delay = types::time_duration::from_secs(3);
            match stream.set_read_timeout(Some(delay)) {
                Ok(_) => {}
                Err(err) => println!("Error timeout: {}", err),
            }
        }

        fn send_request(&self, 
                stream: &mut types::tcp_stream, 
                word: &str) {
            let request = format!("DEFINE * {}\r\n", word);
            match stream.write_all(request.as_bytes()) {
                Ok(_) => {}
                Err(err) => println!("Error request: {}", err),
            }
        }

        fn read_response(&self, 
                stream: &mut types::tcp_stream) {
            let stream_clone = stream.try_clone().unwrap();
            let mut reader = types::buffer_reader::new(stream_clone);
            let mut response = String::new();

            match reader.read_line(&mut response) {
                Ok(0) => {
                    println!("Server sent no response.");
                }
                Ok(_) => {
                    print!("{}", response);
                    self.read_remaining_lines(&mut reader);
                }
                Err(e) if e.kind() == types::error_kind::WouldBlock => {
                    println!("Server timeout.");
                }
                Err(e) => {
                    println!("Error read: {}", e);
                }
            }
        }

        fn read_remaining_lines(&self, 
                reader: &mut types::buffer_reader<types::tcp_stream>) {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        print!("{}", line);
                        if line.starts_with("250") || 
                                line.starts_with("552") {
                            break;
                        }
                    }
                    Err(e) if e.kind() == types::error_kind::WouldBlock => {
                        println!("Server timeout.");
                        break;
                    }
                    Err(e) => {
                        println!("Error reading: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
