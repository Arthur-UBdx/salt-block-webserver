use std::net::{SocketAddr, Ipv4Addr, IpAddr, TcpStream};
use std::{fs, str, io::prelude::*, io::BufReader};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(unused_imports)]
use log::{debug, info, warn, error};

use crate::script_runner::*;
use crate::database_utils::Database;

/// The response which is sent to the client if there was an error in the [handle_connection] function or any function called by it.
/// 
/// Defaults to:
/// ```text
/// HTTP/1.1 500 INTERNAL SERVER ERROR
/// Content-Length: 188
///
/// {
///     "status":"error",
///     "status_code":"500",
///     "message":"internal server error",
///     "result":["There was an internal server error, if the issue persists please contact support."]
/// }
/// ```
static ERR500: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 188\r\n\r\n{\n    \"status\":\"error\",\n    \"status_code\":\"500\",\n    \"message\":\"internal server error\",\n    \"result\":[\"There was an internal server error, if the issue persists please contact support.\"]\n}";
/// Sends back to the client `stream` the message `msg`
///
/// # Example
/// ```
/// let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
/// let message = "HTTP/1.1 200 OK";
/// send!(listener.incoming(), message);
/// ```
/// this will send and OK message to the client
macro_rules! send {
    ($stream: expr, $msg:expr) => {
        {
            match $stream.write_all($msg) {
                Ok(_) => (),
                Err(e) => {info!("Error when writing data to stream: {}", e);},
            }
            match $stream.flush() {
                Ok(_) => (),
                Err(e) => {info!("Error when flushing data to client: {}", e);},
            };
            return;
        }
    };
}

/// Handles the incoming HTTP request
/// Parses the HTTP request, gets the page in the database, runs the script associated or returns the html or json files
/// 
/// # Example:
/// ```
/// let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
/// handle_connection(listener.incoming(), "database.db");
/// ```
/// this will send back to the client the result of the HTTP request made to `127.0.0.1:7878` by said client
pub fn handle_connection(mut stream: TcpStream, database_filepath: &str) {
    let database = Database::new(database_filepath);
    let incoming_request = match IncomingRequest::parse_request(&stream){
        ParsedRequest::Ok(v) => v,
        ParsedRequest::Empty => {return},
        ParsedRequest::BadRequest => {
            info!("An invalid request has been formulated by {}", stream.peer_addr().unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)));
            let mut http_response = match database.get_error(HTTPCode::Err400, &IncomingRequest::new()) {
                ServerStatus::Ok(Some(v)) => v,
                _ => {send!(stream, ERR500.as_bytes());}
            };
            let response = http_response.prepare_response();
            send!(stream, &response);
        }
    };

    debug!("{}\n", incoming_request.as_json());

    let http_code = match database.match_request(&incoming_request) {
        ServerStatus::Ok(v) => v,
        ServerStatus::InternalError => {
            send!(stream, ERR500.as_bytes())
        },
    };

    let response = match http_code {
        HTTPCode::Ok200(v) => {match HTTPResponse::from_matched_request(v, *incoming_request) {
            ServerStatus::Ok(mut v) => v.prepare_response(),
            _ => {send!(stream, ERR500.as_bytes());}
        }},
        _ => {
            let mut http_response = match database.get_error(http_code, &incoming_request) {
                ServerStatus::Ok(Some(v)) => v,
                _ => {send!(stream, ERR500.as_bytes());}
            };
            http_response.prepare_response()
        }
    };
    match std::str::from_utf8(&response) {
        Ok(v) => debug!("{:?}", v),
        Err(_) => debug!("{:?}", response),
    }

    send!(stream, &response);
}

//--//

/// An enum containing the path, headers and content from the incoming request
#[derive(Debug)]
#[allow(dead_code)]
pub struct IncomingRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    _version: String,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    body: String,
}

impl IncomingRequest {
    /// Creates a new empty IncomingRequest object
    pub fn new() -> IncomingRequest {
        IncomingRequest {
            method: String::new(), 
            path: String::new(),
            query: HashMap::new(), 
            _version: String::new(),
            headers: HashMap::new(), 
            cookies: HashMap::new(), 
            body: String::new()}
    }
    /// Parses an HTTP request and returns a [ParsedRequest], also see [IncomingRequest]
    ///
    /// # Example
    ///
    /// ```text
    /// POST /page?key1=value1&key2=value2 HTTP/1.1
    /// First-Header: Value
    /// Content-Length: 31
    /// Content-Type: application/json 
    ///
    /// {
    ///     "body":["thing1", "thing2"]
    /// }
    /// ```
    /// will be converted to:
    /// ```
    /// IncomingRequest {
    ///     method: "POST",
    ///     path: "/page",
    ///     query: {"key1":"value1", "key2", "value2"}
    ///     _version: "1.1",
    ///     headers: {"First-Header":"Value", "Content-Length":"31", "Content-Type":"application/json"}
    ///     body: "{\n\t"body":["thing1", "thing2"]\n}"
    /// }
    pub fn parse_request(mut stream: &TcpStream) -> ParsedRequest {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        match buf_reader.read_line(&mut request_line) {
            Ok(_) => (),
            Err(_) => {
                error!("Error when reading request line from ip {}:\n{:?}",
                    stream.peer_addr().unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 80)),
                    stream);
                return ParsedRequest::BadRequest;
            },
        };

        let (method, uri, version) = match request_line.split_once(' ') {
            Some((method, rest)) => {
                let (uri, version) = rest.split_once(' ').unwrap_or((rest, "HTTP/1.1"));
                (method, uri, version)
            }
            None => return ParsedRequest::Empty,
        };
        let (path, query_map): (String, HashMap<String, String>) = match uri.split_once('?') {
            Some((path, query)) => {
                (path.to_string(), parse_hashmap(query, "&", "="))
            }
            None => (uri.to_string(), HashMap::new()),
        };
        let mut headers_map: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).unwrap();
            if line.as_str() == "\r\n" {break}

            let mut header_splitted = line.split(':');
            let key = String::from(header_splitted.next().unwrap_or("").trim_end());
            let value = String::from(header_splitted.next().unwrap_or("").trim_end());
            headers_map.insert(key.to_lowercase(), value);
        }
        let cookie_map = match headers_map.get("cookie") {
            Some(s) => parse_hashmap(s, ";", "="),
            None => HashMap::new(),
        };
        let mut body = String::new();
        match headers_map.get("content-length") {
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
        let incoming = IncomingRequest {
            method: method.to_string(), 
            path,
            query: query_map, 
            _version: version.trim().to_string(),
            headers: headers_map, 
            cookies: cookie_map, 
            body};

        ParsedRequest::Ok(Box::new(incoming))
    }

    /// Converts the incoming request to a json String in the following format:
    /// ```json
    /// {
    ///     "method":"GET",
    ///     "path":"/index",
    ///     "query":{"key1":"value1", "key2","value2"},
    ///     "version":"1.1",
    ///     "headers":{"accept-language":"en-US,en;q=0.9","cookie":"sessionID=9999;cookie2=hello"},
    ///     "cookies":{"sessionID":"9999","cookie2":"hello"},
    ///     "body":"",
    /// }
    /// ```
    pub fn as_json(&self) -> String {
        format!("{{
            \"method\":\"{}\",
            \"path\":\"{}\",
            \"query\":{},
            \"version\":\"{}\",
            \"headers\":{},
            \"cookies\":{},
            \"body\":\"{}\"
        }}", 
        self.method,
        self.path,
        format!("{:?}", self.query).replace(' ', ""),
        self._version,
        format!("{:?}", self.headers).replace(' ', ""),
        format!("{:?}", self.cookies).replace(' ', ""),
        self.body)
    }
}

/// An enum used by the [IncomingRequest::parse_request] method to handle empty and invalid requests
pub enum ParsedRequest {
    Ok (Box<IncomingRequest>),
    Empty,
    BadRequest,
}

/// An enum to store common HTTP error codes and OK for the [handle_connection] function
pub enum HTTPCode {
    Ok200 (MatchedRequest),
    Err400,
    Err401,
    Err403,
    Err404,
}

/// An enum to handle errors and prevent the threads from panicking, most functions in `request_handler.rs` uses this enum.
/// The [handle_connection] function will send an Error 500 to the client if one of the functions returns `InternalError`.
#[derive(Debug)]
pub enum ServerStatus<T> {
    Ok (T),
    InternalError,
}

/// An enum to handle user authentication by the server, if the auth succeeds, the [Database::auth_user] function will
/// returns `Ok(name, n)` where `n` is the auth level of the user.
///
/// # Auth levels:
/// ```text
/// 0: not logged in
/// 255: admin
/// ```
#[derive(Debug)]
pub enum UserAuth {
    Ok (u8),
    ErrAuth,
}

impl Database {
    /// This function is to find the information (path, page/script filepath, auth level needed and query parameters) in the database and returns a [MatchedRequest]
    pub fn match_request(&self, incoming: &IncomingRequest) -> ServerStatus<HTTPCode> {
        let table = &format!("requests_{}", incoming.method.to_lowercase());
        let key_column = "path";
        let key = &incoming.path;

        let request_result = match self.request_row(table, key_column, key) {
            Ok(v) => v,
            Err(_) => {return ServerStatus::InternalError}, 
        };
        if request_result.is_empty() {
            return ServerStatus::Ok(HTTPCode::Err404);
        }
        let path = String::from(request_result.get("path").unwrap());
        let callback = String::from(request_result.get("callback").unwrap());
        let auth_level: u8 = match request_result.get("auth_level").unwrap().parse::<u8>() {
            Ok(v) => v,
            _ => 255,
        };
        let parameters: Vec<String> = match request_result.get("params") {
            Some(v) => match v.as_str() {
                "" => Vec::new(),
                _ => v.split(';').map(|s| s.trim().to_string()).collect(),
            },
            None => {return ServerStatus::InternalError;}
        };
        if !parameters.is_empty() {
            for key in &parameters {
                if !incoming.query.contains_key(key) {
                    return ServerStatus::Ok(HTTPCode::Err400);
                }
            }
        }
        if auth_level > 0 {
            let user_auth_level = match self.auth_user(incoming) {
                ServerStatus::Ok(v) => match v {
                    UserAuth::Ok(v) => v,
                    UserAuth::ErrAuth => {return ServerStatus::Ok(HTTPCode::Err401)},
                },
                _ => {error!("Error when trying to get user auth level");
                    return ServerStatus::InternalError},
            };

            if user_auth_level < auth_level {
                return ServerStatus::Ok(HTTPCode::Err403);
            }
        }
        ServerStatus::Ok(HTTPCode::Ok200(MatchedRequest {path, callback, auth_level, params: parameters}))
    }

    /// This function will look in the database for a valid `sessionID` found in the [IncomingRequest]'s cookies field and will return the auth_level of this user.
    fn auth_user(&self, incoming: &IncomingRequest) -> ServerStatus<UserAuth> {
        let session_id = match incoming.cookies.get("sessionID") {
            Some(v) => v,
            None => {return ServerStatus::Ok(UserAuth::ErrAuth)}
        };
        let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("ERROR: TIME WENT BACKWARDS").as_secs();
        let result = match self.request_row("users", "sessionID", session_id) {
            Ok(v) => v,
            Err(e) => {error!("{}", e); return ServerStatus::InternalError;}, 
        };
        let auth_level = match result.get("auth_level") {
            Some(v) => match v.parse::<u8>() {
                Ok(v) => ServerStatus::Ok(UserAuth::Ok(v)),
                Err(e) => {error!("{}", e); return ServerStatus::InternalError;}
            },
            None => ServerStatus::Ok(UserAuth::ErrAuth)
        };
        let expires = match result.get("sessionExpires") {
            Some(v) => match v.parse::<u64>() {
                Ok(v) => v,
                Err(e) => {error!("{}", e); return ServerStatus::InternalError;},
            },
            None => {return ServerStatus::Ok(UserAuth::ErrAuth);},
        };
        if expires <= time_now {
            return ServerStatus::Ok(UserAuth::ErrAuth);
        }
        auth_level
    }

    ///This function will look in the database in the `errors` table for where to find the content to send to the client when an error occurs
    ///
    /// # Example
    /// ```
    /// let database = WebserverDatabase::new("database.db")
    /// let error = HTTPCode::Err401
    /// let content = match database.get_error(error) {
    ///     ServerStatus::Ok(v) => v.unwrap()
    /// }
    pub fn get_error(&self, httpcode: HTTPCode, incoming_request: &IncomingRequest) -> ServerStatus<Option<HTTPResponse>> {
        let error_name = match httpcode {
            HTTPCode::Ok200 (_) => {return ServerStatus::Ok(None)},
            HTTPCode::Err400 => "err400",
            HTTPCode::Err401 => "err401",
            HTTPCode::Err403 => "err403",
            HTTPCode::Err404 => "err404",
        };

        let request_result = match self.request_row("errors", "name", error_name) {
            Ok(v) => v,
            Err(_) => {return ServerStatus::InternalError;},
        };

        let (response_message, page_filepath) = match (request_result.get("response_message"), request_result.get("callback")) {
            (Some(v1), Some(v2)) => (v1, v2),
            _ => {return ServerStatus::InternalError;},
        };

        let error_code: u32 = error_name[3..].parse::<u32>().unwrap(); 
        let mut http_response = HTTPResponse::new(error_code, String::from(response_message));
        http_response.load_contents(String::from(page_filepath), &incoming_request.as_json());

        ServerStatus::Ok(Some(http_response))
    }
}

/// A struct returned by the [Database::match_request] function that searches in the database for a page corresponding to the path requested.
#[allow(dead_code)]
pub struct MatchedRequest {
    path: String, 
    callback: String,
    auth_level: u8,
    params: Vec<String>,
}

/// A struct representing the response that will be sent by the server to the client
pub struct HTTPResponse {
    response_code: u32,
    response_message: String,
    headers: HashMap<String, String>,
    contents: Vec<u8>,
}

impl HTTPResponse {
    ///Creates a new HTTPResponse object
    fn new(response_code: u32, response_message: String) -> HTTPResponse {
        HTTPResponse {response_code, response_message, headers: HashMap::new(), contents: Vec::new()}
    }

    ///Uses the [MatchedRequest] containing the file the user requested and other informations and returns a valid HTTPResponse object
    pub fn from_matched_request(matched_request: MatchedRequest, incoming_request: IncomingRequest) -> ServerStatus<HTTPResponse> {
        let mut http_response = HTTPResponse::new(200, String::from("OK"));
        match http_response.load_contents(matched_request.callback, &incoming_request.as_json()) {
            ServerStatus::Ok(()) => (),
            ServerStatus::InternalError => return ServerStatus::InternalError,
        };
        ServerStatus::Ok(http_response)
    }

    /// Loads/runs the content from a file which path was given
    ///
    /// # Example
    ///
    /// myfile.json:
    /// ```
    /// {
    /// "hello": "world"
    /// }
    /// ```
    /// main.rs:
    /// ```
    /// let response = HTTPResponse::new(200, String::from("OK"))
    /// response.load_contents(String::from("myfile.json"), "");
    /// println!("{}", response.contents);
    /// ```
    fn load_contents(&mut self, filename: String, script_args: &str) -> ServerStatus<()> {
        let result = match filename.split('.').last().unwrap_or("") {
            "py" => run_python(&filename, script_args),
            "js" => run_js(&filename, script_args),
            _ => match fs::read(&filename) {
                Ok(v) => Ok(v),
                _ => Err(format!("Error when loading text file {}", filename)),
            },
        };
        let contents = match result {
            Ok(v) => v,
            Err(e) => {
                error!("Error when accessing content:\n{}", e);
                return ServerStatus::InternalError;
            }
        };
        match filename.split('.').last().unwrap_or("") {
            "py"|"js" => {
                let (code_and_message, headers_and_body): (String, Vec<u8>) = match contents.split_once(&[13u8,10u8]) { //[13u8,10u8] <=> b"\r\n"
                    Some((cm, hb)) => (String::from_utf8_lossy(&cm).to_string(), hb.to_vec()),
                    None => (String::from("200 OK"), contents),
                };
                let (code, message) = code_and_message.split_once(' ').unwrap_or(("200", "OK"));
                let (headers, mut body): (HashMap<String, String>, Vec<u8>) = match headers_and_body.split_once(&[13u8,10u8,13u8,10u8]) { //[13u8,10u8,13u8,10u8] <=> b"\r\n\r\n"
                    Some((h,b)) => (parse_hashmap(&String::from_utf8_lossy(&h), "\r\n", ":"), b),
                    None => (HashMap::new(), headers_and_body),
                };
                if !body.is_empty() && body[..2] == [13u8, 10u8] {body = body[2..].to_vec();}
                self.response_code = code.parse::<u32>().unwrap_or(200u32);
                self.response_message = String::from(message);
                self.set_contents(body);
                self.add_headers(headers);
            },
            _ => self.set_contents(contents),
        };
        ServerStatus::Ok(())
    }

    /// Sets the content from the given string
    /// 
    /// #Example
    /// ```
    /// let response = HTTPResponse::new(200, String::from("OK"))
    /// let content = String::from("Hello World");
    /// response.set_contents("Hello World");
    /// println!("{}", response.contents);
    /// ```
    fn set_contents(&mut self, contents: Vec<u8>) {
        self.contents = contents;
        self.update_content_length();
    }

    /// Updates the `Content-Length` header
    fn update_content_length(&mut self) {
        self.headers.insert("Content-Length".to_string(), format!("{}", self.contents.len()));
    }

    /// Adds headers in the response from a given [HashMap]
    fn add_headers(&mut self, headers: HashMap<String, String>) {
        self.headers.extend(headers);
    }

    /// Converts the [HTTPResponse] back to bytes / [Vec]<u8>
    fn prepare_response(&mut self) -> Vec<u8> {
        let mut headers_fmt = String::new();
        self.headers.iter().for_each(|(k,v)| {
            headers_fmt = format!("{}{}: {}\r\n", headers_fmt, k, v);
        });
        let mut bytes: Vec<u8> = format!("HTTP/1.1 {} {}\r\n{}\r\n", 
            self.response_code,
            self.response_message,
            headers_fmt).as_bytes().to_vec();
        bytes.append(&mut self.contents);
        bytes
    }
}

pub fn parse_hashmap(target: &str, entries_separator: &str, key_value_separator: &str) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    let entries = target.split(entries_separator);
    entries.for_each(|e| {
        if let Some((k,v)) = e.split_once(key_value_separator) {
            result.insert(  k.trim().to_string(),
                            v.trim().to_string());
        }
    });
    result
}

#[cfg(test)]
mod tests {
    use crate::request_handler::*;
    #[test]
    fn test_parse_hashmap() {
        let hashmap_test: HashMap<String, String> = HashMap::from([(String::from("sessionID"), String::from("1")),(String::from("cookie2"), String::from("hello"))]);
        let target = " sessionID=1; cookie2=hello; nonvalid";
        let parsed_hashmap = parse_hashmap(target, ";", "=");
        assert_eq!(hashmap_test,parsed_hashmap);
    }
}

trait SplitOnce {
    fn split_once(&self, delimiter: &[u8]) -> Option<(Vec<u8>, Vec<u8>)>;
}

impl SplitOnce for Vec<u8> {
    fn split_once(&self, delimiter: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        if let Some(index) = self.windows(delimiter.len()).position(|w| w == delimiter) {
            let left = self[..index].to_vec();
            let right = self[index + delimiter.len()..].to_vec();
            Some((left, right))
        } else {
            None
        }
    }
}