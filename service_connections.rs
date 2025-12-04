use std::thread;
use std::io::Read;
use std::sync::mpsc;
use std::net::{TcpStream, TcpListener};

use crate::network::communication_point;

pub fn requests_handler(listener: TcpListener) -> mpsc::Receiver<communication_point::Chat> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            let tx_clone = tx.clone();
            thread::spawn(move || {
                match handle_connection(stream) {
                    Ok(chat) => tx_clone.send(chat).unwrap(),
                    Err(e) => eprintln!("Error handle connection: {e}"),
                }
            });
        }
    });

    rx
}

fn handle_connection(mut stream: TcpStream) -> Result<communication_point::Chat, &'static str> {
    let mut buffer = [0; 1024];
    if let Ok(size) = stream.read(&mut buffer) {
        Ok(communication_point::Chat::convert_to_struct(buffer, size)?)
    } else {
        Err("read has failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::{TcpStream, TcpListener};
    use crate::network::communication_point;

    #[test]
    fn handle_test_connection() {
        let listener = TcpListener::bind("127.0.0.1:7878")
            .expect("Can't bind 127.0.0.1:7878");

        let success = communication_point::Chat {
            ip: String::from("127.0.0.1"),
            port: String::from("7878"),
            chat_id: 0,
            content: String::from("Hello"),
        };

        let success_two = success.clone();
        let unsuccess = communication_point::Chat {
            ip: String::from("0.0.0.0"),
            port: String::from("1111"),
            chat_id: 1,
            content: String::from("Goodbye!"),
        };

        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                match handle_connection(stream) {
                    Ok(result) => assert_eq!(unsuccess/*success_two*/, result),
                    Err(e) => panic!("`handle_connection` has failed: {e}"),
                }
            }
        });

        let mut stream = TcpStream::connect("127.0.0.1:7878")
            .expect("Can't connect to `127.0.0.1:7878`");

        stream.write(success.form_request().as_bytes())
            .expect("Can't send message");
    }

    #[test]
    fn handle_test_request() {
        let listener = TcpListener::bind("127.0.0.1:7979")
            .expect("Can't bind 127.0.0.1:7979");

        let rx = requests_handler(listener);
        let success = communication_point::Chat {
            ip: String::from("127.0.0.1"),
            port: String::from("7979"),
            chat_id: 0,
            content: String::from("Hello"),
        };

        let mut stream = TcpStream::connect("127.0.0.1:7979")
            .expect("Can't connect to 127.0.0.1:7979");

        stream.write(success.form_request().as_bytes())
            .expect("Can't send message");

        assert_eq!(success, rx.recv().unwrap());
    }
}