use std::env;
use rand::Rng;
use std::thread;
use std::process;
use std::sync::mpsc;
use std::time::Duration;
use std::net::TcpStream;
use std::io::{self, Write};
use local_ip_address::local_ip;

use crate::network::communication_point;

pub struct AppAddress {
    pub ip: String,
    pub port: String,
}

impl AppAddress {
    fn get_ip_and_port() -> Self {
        let ip = local_ip().expect("Can't get local ip address");
        let args: Vec<String> = env::args().collect();

        let port = args[1].clone();

        Self {
            ip: ip.to_string(),
            port: port,
        }
    }
}

pub fn get_app_address(port: &String) -> String {
    let ip = local_ip().expect("Can't get local ip address");
    format!("{}:{}", ip, port)
}

pub fn input_control() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    tx.send(input)
                        .expect("`send` has failure");
                },
                Err(e) => {
                    panic!("Failure try to read the line: {e}");
                },
            }
        }
    });

    rx
}

pub fn run(rx: mpsc::Receiver<communication_point::Chat>, rx1: mpsc::Receiver<String>) {
    let mut chats: Vec<communication_point::Chat> = Vec::new();

    loop {
        thread::sleep(Duration::from_secs(1));
        if let Ok(request) = rx.try_recv() {
            chats_update(&mut chats, &request);
        }

        if let Ok(input) = rx1.try_recv() {
            handle_input(input, &rx1, &mut chats);
        }
    }
}

fn get_chat_id(chats: &Vec<communication_point::Chat>) -> u32 {
    loop {
        let chat_id: u32 = rand::rng().random_range(1_000_000..4_000_000_000);
        if !chats.iter().any(|chat| chat.chat_id == chat_id) {
            return chat_id;
        }
    }
}

fn chats_update(chats: &mut Vec<communication_point::Chat>, update: &communication_point::Chat) {
    for chat in &mut *chats {
        if chat.chat_id == update.chat_id {
            chat.content.push_str("\n            ");
            chat.content.push_str(&update.content);
            return;
        }
    }

    let new_chat = communication_point::Chat {
        chat_id: update.chat_id.clone(),
        ip: update.ip.clone(),
        port: update.port.clone(),
        content: update.content.clone(),
    };

    chats.push(new_chat);
}

fn handle_input(input: String, rx1: &mpsc::Receiver<String>, chats: &mut Vec<communication_point::Chat>) {
    let command: Vec<_> = input
        .split_whitespace()
        .collect();

    if command.len() == 0 {
        return;
    }

    if command[0] == "exit" {
        exit();
    } else if command[0] == "send" {
        send(&command, rx1, chats);
    } else if command[0] == "list" {
        list(&chats);
    } else if command[0] == "open" {
        open(&command, chats);
    }
}

fn exit() {
    println!("");
    process::exit(0);
}

fn send(command: &Vec<&str>, rx1: &mpsc::Receiver<String>, chats: &mut Vec<communication_point::Chat>) {
    let mut address = String::new();

    if command.len() != 2 {
        println!("Incorrect count of arguments");
        return;
    } else if command[1] == "new" {
        let input = rx1.recv().unwrap();
        let point: Vec<_> = input.split_whitespace().collect();

        address = format!("{}:{}", point[0], point[1]);

        let mut stream = TcpStream::connect(&address).expect("Can't connect to the address");
        let input = rx1.recv().unwrap();

        let new_chat_id = get_chat_id(&*chats);

        let new_chat = communication_point::Chat {
            chat_id: new_chat_id,
            ip: point[0].to_string(),
            port: point[1].to_string(),
            content: input.clone(),
        };

        chats.push(new_chat);
        let app_address = AppAddress::get_ip_and_port();

        let request = communication_point::Chat {
            chat_id: new_chat_id,
            ip: app_address.ip,
            port: app_address.port,
            content: input,
        };

        stream.write(request.form_request().as_bytes())
            .expect("Can't send request");

            return;
    } else {
        for chat in &mut *chats {
            if chat.chat_id == command[1].parse().unwrap() {
                let ip = chat.ip.clone();
                let port = chat.port.clone();
                address = format!("{}:{}", ip, port);
                break;
            }
        }
    }

    let mut stream = TcpStream::connect(address)
        .expect("Can't connect to `{address}`");

    let input = rx1.recv().unwrap();
    let address = AppAddress::get_ip_and_port();

    let request = communication_point::Chat {
        chat_id: command[1].parse().unwrap(),
        ip: address.ip.clone(),
        port: address.port.clone(),
        content: input.clone(),
    };

    chats_update(chats, &request);

    stream.write(request.form_request().as_bytes())
        .expect("Can't send message");
}

fn list(chats: &Vec<communication_point::Chat>) {
    let mut i = 0;

    for chat in chats {
        println!("chat {}: {}", i, chat.chat_id);
        i += 1;
    }
}

fn open(command: &Vec<&str>, chats: &mut Vec<communication_point::Chat>) {
    if command.len() != 2 {
        println!("Incorrect count of arguments");
        return;
    }

    for chat in chats {
        if chat.chat_id == command[1].parse().unwrap() {
            println!("{}", chat.content);
            return;
        }
    }
}