use websocket;
use reqwest;
use std::process::Command;
use websocket::url::Url;
use serde;  
use std::{thread, time};

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

enum ExpositionError {
    ParseError(websocket::client::ParseError),
    ReqwestError(reqwest::Error),
}

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
    let _handle = Command::new(r"C:\Users\Reilley Pfrimmer\AppData\Local\Discord\app-1.0.9018\Discord.exe") // REMEMBER TO FUCKING CHANGE THIS BEFORE SHIPPING LOL (otherwise it will only work on my computer)
        .arg("--remote-debugging-port=9222")
        .spawn()
        .expect("Failed to open Discord.Exe"); // I'm keeping the handle just for the lols.
    thread::sleep(time::Duration::from_secs(25)); // TODO figure out a better way to detect when discord loads because this is shitty as FUCK
    let webSocketDebuggerUrl: Url = match exposeWebSocketDebuggerUrl() { 
        Ok(a) => a, // Retrieve webSocketDebuggerUrl from json endpoint
        Err(e) => match e {
            ExpositionError::ParseError(e) => panic!("expose: ParseError! {e:?}"),
            ExpositionError::ReqwestError(e) => panic!("expose: ReqwestError! {e:?}"),
        },
    };
    let mut webSocketConnection: websocket::client::sync::Client<websocket::stream::sync::TcpStream> = match buildWebSocketConnection(webSocketDebuggerUrl.clone()) {
        Ok(a) => a, // establish connection with websocket using webSocketDebuggerUrl
        Err(e) => match e {
            ConnectionError::ParseError(e) => panic!("connect: ParseError! {e:?}"),
            ConnectionError::WebSocketError(e) => panic!("connect: WebSocketError! {e:?}"),
        },
    };
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