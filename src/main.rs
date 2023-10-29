use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::fs;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:4221"){
        Ok(listener) => listener,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    match listener.local_addr() {
        Ok(addr) => println!("Server start on {}", addr),
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    }

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                thread::spawn(move || {
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let request_str = String::from_utf8_lossy(&buffer[..]);

    if request_str.contains("GET") {
        get_request(&stream, &request_str);
    } else if request_str.contains("POST") {
        post_request(&stream, &request_str);
    } else {
        println!("Error: invalid request");
        match stream.write("HTTP/1.1 400 Bad Request\r\n\r\n".as_bytes()) {
            Ok(_) => println!("Response sent: \n{:?}", "HTTP/1.1 400 Bad Request\r\n\r\n"),
            Err(e) => println!("Failed sending response: {}", e),
        }
    }
}

fn get_request(mut stream: &TcpStream, request_str: &str) {
    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first_line = lines[0];
    let path: Vec<&str> = first_line.split(" ").collect();
    let pathname = path[1];

    let response = match pathname {
        _task_manager if pathname.contains("/task-manager") || pathname == "/" => {
            println!("GET {}", pathname);

            match fs::read_to_string("src/task-manager/index.html") {
                Ok(contents) => {
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n", contents.len(), contents)
                }

                Err(e) => {
                    println!("Error: {}", e);
                    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                }

            }
        }
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };

    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Response sent: \n{:?}", response),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn post_request(mut stream: &TcpStream, request_str: &str) {
    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first_line = lines[0];
    let path: Vec<&str> = first_line.split(" ").collect();
    let pathname = path[1];
    let body = match request_str.split("\r\n\r\n").last() {
        Some(body) => body.trim_end_matches(char::from(0)),
        None => "",
    };

    let response = match pathname {
        _files if pathname.contains("/files") => {
            println!("{}", body);
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        },
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    };

    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Response sent: \n{:?}", response),
        Err(e) => println!("Failed sending response: {}", e),
    }
}