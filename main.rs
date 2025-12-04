use std::env;
use std::net::TcpListener;

mod network;
use crate::network::service_connections;
use crate::network::shell;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app_address = shell::get_app_address(&args[1]);

    println!("app_address: `{}`", app_address);
    let listener = TcpListener::bind(app_address).expect("Can't bind ip and port `{}`");

    let rx = service_connections::requests_handler(listener);
    let rx1 = shell::input_control();

    shell::run(rx, rx1);
}