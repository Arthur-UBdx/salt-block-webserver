use std::net::TcpListener;

#[allow(unused_imports)]
use log::{LevelFilter, info, warn, error};

use simplelog::{CombinedLogger, WriteLogger, TermLogger, TerminalMode, Config, ColorChoice};
use std::{fs, fs::File, sync::Arc};
use std::env;

mod script_runner;
mod database_utils;
mod request_handler;
mod thread_pool;

use crate::thread_pool::*;
use crate::request_handler::*;

fn main() {
    let server_config_file: String = match env::var("SERVER_CONFIG") {
        Ok(v) => v,
        _ => panic!("Didn't found server config file, the path is stored in the SERVER_CONFIG env variable"),
    };
    let mut config_string = fs::read_to_string(server_config_file).unwrap();
    config_string = config_string.replace("\"", "")
        .replace("\n", "")
        .strip_prefix("{").unwrap().to_string()
        .strip_suffix("}").unwrap().to_string();

    let config = parse_hashmap(config_string.trim(), ",", ":");

    let logging_level: LevelFilter = match env::var("LOG_LEVEL") {
        Ok(v) => match v.trim() {
            "0" => LevelFilter::Error,
            "1" => LevelFilter::Warn,
            "2" => LevelFilter::Info,
            "3" => LevelFilter::Debug,
            _ => LevelFilter::Warn,
        },
        _ => LevelFilter::Warn,
    };
    println!("The logging level is currently set to : {:?}\nChange the value of the environnement variable 'LOG_LEVEL' to change it.", logging_level);

    let logfile = File::create(config.get("log_file").unwrap()).unwrap();
    let tlogger = TermLogger::new(logging_level,Config::default(),TerminalMode::Stdout,ColorChoice::Auto);
    let wlogger = WriteLogger::new(logging_level,Config::default(),logfile);
    CombinedLogger::init(vec![tlogger, wlogger]).unwrap();

    let listener = TcpListener::bind(config.get("ip").unwrap()).unwrap();
    let pool = ThreadPool::new(4);

    let database = Arc::new(config.get("database").unwrap().to_string());
    for stream in listener.incoming() {
        let arc_db = database.clone();
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream, &arc_db);
        });
    };
    println!("shutting down")
}