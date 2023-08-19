use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use web_server_rust::ThreadPool;

fn main() {
    if let Ok(listener) = TcpListener::bind("127.0.0.1:7878") {
        let pool = ThreadPool::new(4);

        for stream in listener.incoming().take(2) {
        //let stream = stream.unwrap();

            if let Ok(ok_stream) = stream {
                pool.execute(|| {
                    handle_connection(ok_stream);
                });
            }
        }

        println!("Shutting down.");
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    //let request_line = buf_reader.lines().next().unwrap().unwrap();
    if let Some(request) = buf_reader.lines().next() {
        match request {
            Ok(request_line) => {
                let (status_line, filename) = match &request_line[..] {
                    "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
                    "GET /sleep HTTP/1.1" => {
                        thread::sleep(Duration::from_secs(5));
                        ("HTTP/1.1 200 OK", "hello.html")
                    }
                    _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
                };
                    
                if let Ok(contents) = fs::read_to_string(filename) {
                    let length = contents.len();
            
                    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
            
                    stream.write_all(response.as_bytes()).unwrap();
                }
            },
            Err(_) => println!("Error while handling connections"),
        }
    };
}