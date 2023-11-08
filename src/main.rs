use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

const TOP_HTML: &str = 
"<!DOCTYPE html>
<html lang='en'>

<head>
    <meta charset='UTF-8' />
    <meta name='viewport' content='width=device-width, initial-scale=1.0' />
    <title>Todo</title>
    <style>
        span {
            display: inline-block;
            width: 200px;
            text-align: left;
        }
        form {
            margin: 4px 0;
        }
    </style>
</head>

<body>
    <h1>To-Do</h1>";

const NO_TODOS_HTML: &str =
"<p>No todos yet</p>";

const TODO_HTML_START: &str =
"<form action='/delete_todo' method='POST'>
    <input type='submit' value='X' />
    <span>";

const TODO_HTML_MIDDLE: &str =
"</span>
    <input type='hidden' name='id' value='";

const TODO_HTML_END: &str =
"' />
</form>";

const FORM_HTML: &str =
"<form action='/add_todo' method='POST'>
    <input type='text' name='todo' />
    <input type='submit' value='Add' />
</form>";

const END_HTML: &str =
"</body>

</html>";

struct Todo {
    id: i32,
    text: String,
}

lazy_static! {
    static ref TODOS: Mutex<Vec<Todo>> = Mutex::new(Vec::new());
}

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:8080"){
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
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let request_str = String::from_utf8_lossy(&buffer[..]);

    let response = match request_str {
        _ if request_str.contains("GET") => get_request(&request_str),
        _ if request_str.contains("POST") => post_request(&request_str),
        _ => {
            println!("Error: invalid request");
            "HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
        },
    };

    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Response sent: \n{:?}", response),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn get_request(request_str: &str) -> String {
    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first_line = lines[0];
    let path: Vec<&str> = first_line.split(" ").collect();
    let pathname = path[1];
    
    println!("GET {}", pathname);

    match pathname {
        _get_todo if pathname == "/" => get_todos_request(),
        _ => "HTTP/1.1 404 Not Found\r\n\r\n404 Page not found\r\n\r\n".to_string(),
    }
}

fn get_todos_request() -> String {
    let todos = match TODOS.lock() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error: {}", e);
            return "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string();
        }
    };
    
    create_http_response(&todos)
}

fn post_request(request_str: &str) -> String {
    let lines: Vec<&str> = request_str.split("\r\n").collect();
    let first_line = lines[0];
    let path: Vec<&str> = first_line.split(" ").collect();
    let pathname = path[1];
    let body = match request_str.split("\r\n\r\n").last() {
        Some(body) => body.trim_end_matches(char::from(0)),
        None => "",
    };

    println!("POST {}", pathname);

    match pathname {
        _add_todo if pathname.contains("/add_todo") => {
            add_todos_request(&body)
        },
        _delete_todo if pathname.contains("/delete_todo") => {
            delete_todos_request(&body)
        },
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    }
}

fn add_todos_request(body : &str) -> String {
    // Get todos
    let mut todos = match TODOS.lock() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error: {}", e);
            return "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string();
        }
    };

    // Create new todo
    let id = todos.len() as i32;
    let body_split: Vec<&str> = body.split("=").collect();

    let todo = Todo {
        id,
        text: body_split[1].to_string(),
    };
    todos.push(todo);

    create_http_response(&todos)    
}

fn delete_todos_request(body: &str) -> String {
    // Get todos
    let mut todos = match TODOS.lock() {
        Ok(todos) => todos,
        Err(e) => {
            println!("Error: {}", e);
            return "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string();
        }
    };

    // Delete todo
    let body_split: Vec<&str> = body.split("=").collect();
    let id = match body_split[1].parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            println!("Error: {}", e);
            return "HTTP/1.1 500 Internal Server Error\r\n\r\n".to_string();
        }
    };
    todos.retain(|todo| todo.id != id);

    create_http_response(&todos)
}

fn create_http_response(todos: &Vec<Todo>) -> String {
    // Create HTML for todos
    let mut todos_html = String::new();
    for todo in todos.iter() {
        let mut todo_html: String = String::new();
        todo_html.push_str(TODO_HTML_START);
        todo_html.push_str(&todo.text);
        todo_html.push_str(TODO_HTML_MIDDLE);
        todo_html.push_str(&todo.id.to_string());
        todo_html.push_str(TODO_HTML_END);

        todos_html.push_str(&todo_html);
    }

    // Create response
    let mut response = String::new();
    response.push_str("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n");
    response.push_str(TOP_HTML);
    if todos_html.is_empty() {
        response.push_str(NO_TODOS_HTML);
    } else {
        response.push_str(&todos_html);
    }
    response.push_str(FORM_HTML);
    response.push_str(END_HTML);

    response
}

