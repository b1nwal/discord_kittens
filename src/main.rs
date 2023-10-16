use websocket;
use reqwest;
use std::process::{Command,Stdio};
use websocket::url::Url;
use serde;  
use std::io;
use std::io::BufRead;

#[derive(serde::Deserialize)]
struct ExpositionResponse {
    description: String,
    devtoolsFrontendUrl: String,
    id: String,
    title: String,
    #[serde(rename="type")]
    r_type: String,
    url: String,
    webSocketDebuggerUrl: String,
}


#[derive(Debug)]
enum ExpositionError {
    ParseError(websocket::client::ParseError),
    ReqwestError(reqwest::Error),
}
#[derive(Debug)]
enum ConnectionError {
    ParseError(websocket::client::ParseError),
    WebSocketError(websocket::WebSocketError),
}

// TODO write a macro to implement these traits 
impl From<reqwest::Error> for ExpositionError {
    fn from(err: reqwest::Error) -> Self {
        ExpositionError::ReqwestError(err)
    }
}
impl From<websocket::client::ParseError> for ExpositionError {
    fn from(err: websocket::client::ParseError) -> Self {
        ExpositionError::ParseError(err)
    }
}
impl From<websocket::client::ParseError> for ConnectionError {
    fn from(err: websocket::client::ParseError) -> Self {
        ConnectionError::ParseError(err)
    }
}
impl From<websocket::WebSocketError> for ConnectionError {
    fn from(err: websocket::WebSocketError) -> Self {
        ConnectionError::WebSocketError(err)
    }
}

fn main() {
    let mut handle = Command::new(r"C:\Users\Reilley Pfrimmer\AppData\Local\Discord\app-1.0.9019\Discord.exe") // REMEMBER TO FUCKING CHANGE THIS BEFORE SHIPPING LOL (otherwise it will only work on my computer)
        .arg("--remote-debugging-port=9222")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to open Discord.Exe");
    let handle_stdout = handle.stdout.take().expect("Stdout Error");
    let mut reader = io::BufReader::new(handle_stdout).lines();
    loop {
        if let Some(x) = reader.next() {
            if let Ok(line) = x {
               if line.contains("splashScreen.pageReady") {
                   break;
               }
            }
        }
    }
    let webSocketDebuggerUrl: Url = exposeWebSocketDebuggerUrl().expect("Exposition Error");
    let mut webSocketConnection: websocket::client::sync::Client<websocket::stream::sync::TcpStream> = buildWebSocketConnection(webSocketDebuggerUrl).expect("Websocket Error");
    // This is the payload, edit this
    let injectorJSONDataPayload: websocket::Message = websocket::Message::text(r#"{"id": 1, "method": "Runtime.evaluate", "params": {"contextId": 1, "doNotPauseOnExceptionsAndMuteConsole": false, "expression": "document.writeln(\"We are now coding\")", "generatePreview": false, "includeCommandLineAPI": true, "objectGroup": "console", "returnByValue": false, "userGesture": true}}"#);
    webSocketConnection.send_message(&injectorJSONDataPayload).expect("Send Error");
    let response = webSocketConnection.recv_message().expect("Recv Error"); // probably won't need this
}

fn exposeWebSocketDebuggerUrl() -> Result<Url, ExpositionError> {
    let json: Vec<ExpositionResponse> = reqwest::blocking::get("http://localhost:9222/json/list")?
        .json()?;
    Ok(Url::parse(&json[0].webSocketDebuggerUrl)?)
}

fn buildWebSocketConnection(webSocketDebuggerUrl: Url) -> Result<websocket::client::sync::Client<websocket::stream::sync::TcpStream>, ConnectionError> {
    let client: websocket::client::sync::Client<websocket::stream::sync::TcpStream> = websocket::ClientBuilder::new(&webSocketDebuggerUrl.into_string())?
        .connect_insecure()?;
    Ok(client)
}
