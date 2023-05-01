use std::net::TcpStream;
use std::{fs, str, io::prelude::*, io::BufReader};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use hex;

#[allow(unused_imports)]
use log::{debug, info, warn, error};

use crate::script_runner::*;

// mod script_runner;
// use crate::script_runner::*;

static ERR500: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 188\r\n\r\n{\n    \"status\":\"error\",\n    \"status_code\":\"500\",\n    \"message\":\"internal server error\",\n    \"result\":[\"There was an internal server error, if the issue persists please contact support.\"]\n}";

macro_rules! send {
    ($stream: expr, $msg:expr) => {
        {
            if $msg.len() > 500 {debug!("Sending Response \n\"{}\"", $msg);}
            $stream.write_all($msg.as_bytes()).unwrap();
            $stream.flush().unwrap();
            return;
        }
    };
}

pub fn handle_connection(mut stream: TcpStream, database_filepath: &str) {
    let database = WebserverDatabase::new(database_filepath);
    let ret = match IncomingRequest::parse_request(&stream) {
        ParsedRequest::Ok(v) => v,
        _ => {error!("bad request");
            return;
        },
    };
    println!("{:#?}", ret);
    return;

    // let incoming_request = match IncomingRequest::parse_request(&stream){
    //     ParsedRequest::Ok(v) => v,
    //     ParsedRequest::Empty => {return},
    //     ParsedRequest::BadRequest => {
    //         warn!("An invalid request has been formulated by {}", stream.peer_addr());
    //         let http_response = match database.get_error(HTTPCode::Err400) {
    //             ServerStatus::Ok(Some(v)) => v,
    //             _ => {send!(stream, ERR500);}
    //         };
    //         let response = format!("HTTP/1.1 400 BAD REQUEST\r\nContent-Length: {}\r\n\r\n{}", http_response.content_length, http_response.contents);
    //         send!(stream, response);
    //     }
    // };
    // debug!("{:#?}", incoming_request);

    // let http_code = match database.match_request(incoming_request) {
    //     ServerStatus::Ok(v) => v,
    //     ServerStatus::InternalError => {
    //         send!(stream, ERR500)
    //     },
    // };

    // let response = match http_code {
    //     HTTPCode::Ok200(v) => {match HTTPResponse::from_matched_request(v) {
    //         ServerStatus::Ok(v) => v.prepare_response(),
    //         _ => {send!(stream, ERR500);}
    //     }},
    //     _ => {
    //         let http_response = match database.get_error(http_code) {
    //             ServerStatus::Ok(Some(v)) => v,
    //             _ => {send!(stream, ERR500);}
    //         };
    //         http_response.prepare_response()
    //     }
    // };
    
    // debug!("{}", response);
    // send!(stream, response);
}

//--

enum HTTPCode {
    Ok200 (MatchedRequest),
    Err400,
    Err401,
    Err403,
    Err404,
    Err405,
}

enum ServerStatus<T> {
    Ok(T),
    InternalError,
}

enum UserAuth {
    Ok (u8),
    ErrAuth,
    ErrNotFound,
}

struct MatchedRequest {
    path: String, 
    callback: String,
    auth_level: u8,
    params: Vec<String>,
}

struct HTTPResponse {
    response_code: u32,
    response_message: String,
    contents: String,
    content_length: usize,
}

impl HTTPResponse {
    pub fn new(response_code: u32, response_message: String) -> HTTPResponse {
        let http_response = HTTPResponse {response_code, response_message, contents: String::new(), content_length: 0usize};
        http_response
    }

    pub fn from_matched_request(matched_request: MatchedRequest) -> ServerStatus<HTTPResponse> {
        let mut http_response = HTTPResponse::new(200, String::from("OK"));
        match http_response.load_contents(matched_request.callback) {
            ServerStatus::Ok(()) => (),
            ServerStatus::InternalError => return ServerStatus::InternalError,
        };
        ServerStatus::Ok(http_response)
    }

    fn load_contents(&mut self, filename: String) -> ServerStatus<()> {
        let result = match (&filename).split('.').last().unwrap_or("") {
            "py" => run_python(&filename),
            "js" => run_js(&filename),
            _ => match fs::read_to_string(&filename) {
                Ok(v) => Ok(v),
                _ => Err(String::from("Error when loading text file")),
            },
        };
        let contents = match result {
            Ok(v) => v,
            Err(e) => {
                error!("Error when accessing content:\n{}", e);
                return ServerStatus::InternalError;
            }
        };
        self.set_contents(contents);
        self.update_content_length();
        ServerStatus::Ok(())
    }

    fn set_contents(&mut self, contents: String) {
        self.contents = contents;
        self.update_content_length();
    }

    #[allow(dead_code)]
    pub fn send_response(&self, mut stream: TcpStream) -> ServerStatus<()> {
        let response = format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}", 
            self.response_code,
            self.response_message,
            self.content_length,
            self.contents);

        match stream.write_all(response.as_bytes()) {
            Ok(()) => (),
            _ => {error!("Error when writing buffer into stream");
                return ServerStatus::InternalError},
        };

        match stream.flush() {
            Ok(()) => (),
            _ => {error!("Error when sending stream");
                return ServerStatus::InternalError},
        };

        ServerStatus::Ok(())
    }

    fn update_content_length(&mut self) {
        self.content_length = self.contents.len();
    }

    fn prepare_response(&self) -> String {
        format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}", 
            self.response_code,
            self.response_message,
            self.content_length,
            self.contents)
    }
}

struct WebserverDatabase {
    filepath: String,
}

impl WebserverDatabase {
    fn new(filepath: &str) -> WebserverDatabase {
        WebserverDatabase {filepath: String::from(filepath)}
    }

    fn match_request(&self, incoming: IncomingRequest) -> ServerStatus<HTTPCode> {
        let connection = match sqlite::open(&self.filepath) {
            Ok(v) => v,
            _ => {
                error!("Error when connecting to SQL database");
                return ServerStatus::InternalError}
        };

        let mut statement = match connection.prepare(format!("SELECT * FROM requests_{} WHERE path = ?", incoming.method)) {
            Ok(v) => v,
            _ => {
                error!("Error when executing SQL request");
                return ServerStatus::InternalError}
        };
        
        match statement.bind((1, incoming.path.as_str())) {
            Ok(v) => v,
            _ => {
                error!("Error when executing SQL request");
                return ServerStatus::InternalError}
        };
        if let sqlite::State::Done = statement.next().unwrap() {
            return ServerStatus::Ok(HTTPCode::Err404);
        }

        let callback: String = statement.read::<String, usize>(1).unwrap();
        let auth_level: u8 = match statement.read::<String, usize>(2).unwrap().parse::<u8>() {
            Ok(v) => v,
            _ => 255,
        };
        let parameters: Vec<String> = statement.read::<String, usize>(3).unwrap()
        .split(";")
        .map(|s| s.to_string())
        .collect();
        
        if auth_level > 0 {
            let (username, password): (String, String) = match (incoming.query.get("username"), incoming.query.get("password")) {
                (Some(u), Some(p)) => (String::from(u), String::from(p)),
                _ => return ServerStatus::Ok(HTTPCode::Err401) 
            };

            let user_auth_level = match self.auth_user(username, password) {
                ServerStatus::Ok(v) => match v {
                    UserAuth::Ok(v) => v,
                    _ => {return ServerStatus::Ok(HTTPCode::Err401)},
                },
                _ => {error!("Error when trying to get user auth level");
                    return ServerStatus::InternalError},
            };

            if user_auth_level < auth_level {
                return ServerStatus::Ok(HTTPCode::Err403);
            }
        }
        ServerStatus::Ok(HTTPCode::Ok200(MatchedRequest {path: incoming.path, callback: callback, auth_level: auth_level, params: parameters}))
    }

    fn auth_user(&self, username: String, password: String) -> ServerStatus<UserAuth> {
        let connection = match sqlite::open(&self.filepath) {
            Ok(v) => v,
            _ => {
                error!("Error when connecting to SQL database");
                return ServerStatus::InternalError;
            },
        };

        let sql_request = "SELECT * FROM users WHERE username = ?";
        let mut statement = match connection.prepare(sql_request) {
            Ok(v) => v,
            _ => {
                error!("Error when executing SQL request:\n {}", sql_request);
                return ServerStatus::InternalError;
            },
        };
        match statement.bind((1, username.as_str())) {
            Ok(v) => v,
            _ => {
                error!("Error when executing SQL request");
                return ServerStatus::InternalError}
        };

        if let sqlite::State::Done = statement.next().unwrap() {
            return ServerStatus::Ok(UserAuth::ErrNotFound);
        }

        let correct_hash = statement.read::<String, usize>(1).unwrap();
        let auth_level = match statement.read::<String, usize>(3).unwrap().parse::<u8>() {
            Ok(v) => v,
            _ => 0,
        };

        let mut auth_hash = Sha256::new();
        auth_hash.update(format!("{}{}", username, password).as_bytes());
        let result = hex::encode(auth_hash.finalize());

        debug!("Hash comparaison:\ncorrect {}\nactual  {}\n", correct_hash, result);

        if result == correct_hash {
        ServerStatus::Ok(UserAuth::Ok(auth_level))
        } else {   
            ServerStatus::Ok(UserAuth::ErrAuth)
        }

    }

    pub fn get_error(&self, httpcode: HTTPCode) -> ServerStatus<Option<HTTPResponse>> {
        let error_name = match httpcode {
            HTTPCode::Ok200 (_) => {return ServerStatus::Ok(None)},
            HTTPCode::Err400 => "err400",
            HTTPCode::Err401 => "err401",
            HTTPCode::Err403 => "err403",
            HTTPCode::Err404 => "err404",
            HTTPCode::Err405 => "err405",
        };

        let connection = match sqlite::open(&self.filepath) {
            Ok(v) => v,
            _ => {
                error!("Error when connecting to SQL database");
                return ServerStatus::InternalError;
            },
        };

        let sql_request = &format!("SELECT * FROM errors WHERE name = '{}'", error_name);
        let mut statement = match connection.prepare(sql_request) {
            Ok(v) => v,
            _ => {
                error!("Error when executing SQL request: \n{}", sql_request);
                return ServerStatus::InternalError;
            },
        };

        if let sqlite::State::Done = statement.next().unwrap() {
            error!("Error code '{}' not found in database", error_name);
            return ServerStatus::InternalError;
        }

        let response_message = statement.read::<String, usize>(1).unwrap();
        let page_filepath = statement.read::<String, usize>(2).unwrap();

        let error_code: u32 = error_name[3..].parse::<u32>().unwrap(); 
        let mut http_response = HTTPResponse::new(error_code, String::from(response_message));
        http_response.load_contents(String::from(page_filepath));

        ServerStatus::Ok(Some(http_response))
    }
}

enum ParsedRequest {
    Ok(IncomingRequest),
    Empty,
    BadRequest,
}

#[derive(Debug)]
struct IncomingRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    _version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl IncomingRequest {
    pub fn parse_request(mut stream: &TcpStream) -> ParsedRequest {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        buf_reader.read_line(&mut request_line).unwrap();
        
        let (method, uri, version) = match request_line.split_once(' ') {
            Some((method, rest)) => {
                let (uri, version) = rest.split_once(' ').unwrap();
                (method, uri, version)
            }
            None => return ParsedRequest::BadRequest,
        };
        
        let mut query_map: HashMap<String, String> = HashMap::new();
        info!("URI = {}", uri);
        let (path, query_map): (String, HashMap<String, String>) = match uri.split_once('?') {
            Some((path, query)) => {
                let parts = query.split('&');
                parts.for_each(|s| {
                    let (key, value) = s.split_once('=').unwrap_or(("",""));
                    query_map.insert(key.to_string(), value.to_string());
                });
                (path.to_string(), query_map)
            }
            None => (uri.to_string(), HashMap::new()),
        };
        
        
        let mut headers_map: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).unwrap();
            match line.as_str() {
                "\r\n" => break,
                _ => {}
            }
            let mut header_splitted = line.split(":");
            let key = String::from(header_splitted.next().unwrap().trim_end());
            let value = String::from(header_splitted.next().unwrap().trim_end());
            headers_map.insert(key, value);
        }

        let mut body = String::new();
        let content_length = match headers_map.get("Content-Length") {
            None => 0usize,
            Some(v) => {
                match v.trim().parse::<usize>() {
                    Ok(v) => {
                        let mut body_buffer = vec![0; v];
                        buf_reader.read_exact(&mut body_buffer).unwrap();
                        body = String::from_utf8_lossy(&body_buffer).to_string();
                        v
                    },
                    _ => return ParsedRequest::BadRequest,
                }
            }
        };

        let incoming = IncomingRequest {method: method.to_string(), path: path, query: query_map, _version: version.to_string(), headers: headers_map, body: body};
        ParsedRequest::Ok(incoming)
    }
}