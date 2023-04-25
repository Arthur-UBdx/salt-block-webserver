use std::net::TcpListener;

use log::{LevelFilter, info, warn, error};
use simplelog::{CombinedLogger, WriteLogger, TermLogger, TerminalMode, Config, ColorChoice};
use std::fs::File;
use std::env;

mod request_handler;
mod thread_pool;

use crate::thread_pool::*;
use crate::request_handler::*;

static DATABASE: &str = "database.db";
static LOG_FILE: &str = "log/server.log";

fn main() {
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

    let logfile = File::create(LOG_FILE).unwrap();
    let tlogger = TermLogger::new(logging_level,Config::default(),TerminalMode::Stdout,ColorChoice::Auto);
    let wlogger = WriteLogger::new(logging_level,Config::default(),logfile);
    CombinedLogger::init(vec![tlogger, wlogger]).unwrap();

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream, DATABASE);
        });
    };
    println!("shutting down")
}