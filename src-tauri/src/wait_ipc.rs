use crate::CliArgs;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub(crate) struct WaitResponder {
    endpoint: String,
    token: String,
}

impl WaitResponder {
    pub(crate) fn new(endpoint: String, token: String) -> Self {
        Self { endpoint, token }
    }

    fn send(self, result: Result<String, String>) {
        let endpoint = match self.endpoint.parse::<SocketAddr>() {
            Ok(endpoint) if endpoint.ip().is_loopback() => endpoint,
            _ => {
                log::error!("Refusing invalid non-loopback --wait response endpoint");
                return;
            }
        };
        let response = match result {
            Ok(text) => WaitResponse {
                token: self.token,
                text: Some(text),
                error: None,
            },
            Err(error) => WaitResponse {
                token: self.token,
                text: None,
                error: Some(error),
            },
        };

        match TcpStream::connect(endpoint) {
            Ok(mut stream) => match serde_json::to_vec(&response) {
                Ok(payload) => {
                    if let Err(error) = stream.write_all(&payload) {
                        log::error!("Failed to send --wait response: {error}");
                    }
                }
                Err(error) => log::error!("Failed to serialize --wait response: {error}"),
            },
            Err(error) => log::error!("Failed to connect to --wait client: {error}"),
        }
    }

    pub(crate) fn send_error(self, error: &str) {
        self.send(Err(error.to_string()));
    }
}

#[derive(Default)]
pub(crate) struct WaitResponseState(Mutex<Option<WaitResponder>>);

impl WaitResponseState {
    pub(crate) fn register(&self, responder: WaitResponder) -> Result<(), String> {
        let mut pending = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if pending.is_some() {
            let error = "another --wait recording is already active".to_string();
            drop(pending);
            responder.send_error(&error);
            return Err(error);
        }
        *pending = Some(responder);
        Ok(())
    }

    pub(crate) fn complete(&self, result: Result<String, String>) -> bool {
        let responder = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(responder) = responder {
            responder.send(result);
            true
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WaitResponse {
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn decode_response(payload: &[u8], expected_token: &str) -> Result<String, String> {
    let response: WaitResponse = serde_json::from_slice(payload)
        .map_err(|error| format!("invalid transcription response: {error}"))?;
    if response.token != expected_token {
        return Err("invalid transcription response token".to_string());
    }
    if let Some(error) = response.error {
        return Err(error);
    }
    Ok(response.text.unwrap_or_default())
}

pub(crate) fn run_client(args: &CliArgs) -> i32 {
    let listener = match TcpListener::bind(("127.0.0.1", 0)) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("error: cannot create --wait response listener: {error}");
            return 1;
        }
    };
    let endpoint = match listener.local_addr() {
        Ok(address) => address.to_string(),
        Err(error) => {
            eprintln!("error: cannot inspect --wait response listener: {error}");
            return 1;
        }
    };
    let token = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("error: cannot locate Handy executable: {error}");
            return 1;
        }
    };

    let mut command = Command::new(executable);
    command
        .arg("--start-recording")
        .arg("--wait")
        .arg("--wait-endpoint")
        .arg(&endpoint)
        .arg("--wait-token")
        .arg(&token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if args.post_process {
        command.arg("--post-process");
    }
    if args.debug {
        command.arg("--debug");
    }

    if let Err(error) = command.spawn() {
        eprintln!("error: cannot launch Handy --wait request: {error}");
        return 1;
    }

    let (mut stream, _) = match listener.accept() {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("error: failed waiting for transcription: {error}");
            return 1;
        }
    };
    let mut payload = Vec::new();
    if let Err(error) = stream.read_to_end(&mut payload) {
        eprintln!("error: failed reading transcription response: {error}");
        return 1;
    }
    let text = match decode_response(&payload, &token) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    if args.json {
        println!("{}", serde_json::json!({ "text": text }));
    } else {
        println!("{text}");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listener() -> TcpListener {
        TcpListener::bind(("127.0.0.1", 0)).expect("bind test listener")
    }

    fn responder(listener: &TcpListener, token: &str) -> WaitResponder {
        WaitResponder::new(
            listener.local_addr().expect("listener address").to_string(),
            token.to_string(),
        )
    }

    fn receive(listener: &TcpListener) -> Vec<u8> {
        let (mut stream, _) = listener.accept().expect("accept response");
        let mut payload = Vec::new();
        stream.read_to_end(&mut payload).expect("read response");
        payload
    }

    #[test]
    fn successful_response_is_delivered_once() {
        let listener = listener();
        let state = WaitResponseState::default();
        state
            .register(responder(&listener, "request-1"))
            .expect("register responder");

        assert!(state.complete(Ok("hello\nworld".into())));
        assert_eq!(
            decode_response(&receive(&listener), "request-1").as_deref(),
            Ok("hello\nworld")
        );
        assert!(!state.complete(Err("already consumed".into())));
    }

    #[test]
    fn concurrent_waiter_is_rejected_without_consuming_first() {
        let first_listener = listener();
        let second_listener = listener();
        let state = WaitResponseState::default();
        state
            .register(responder(&first_listener, "first"))
            .expect("register first responder");

        assert!(state
            .register(responder(&second_listener, "second"))
            .is_err());
        assert_eq!(
            decode_response(&receive(&second_listener), "second").unwrap_err(),
            "another --wait recording is already active"
        );

        assert!(state.complete(Ok("first result".into())));
        assert_eq!(
            decode_response(&receive(&first_listener), "first").as_deref(),
            Ok("first result")
        );
    }

    #[test]
    fn response_token_is_verified() {
        let payload = serde_json::to_vec(&WaitResponse {
            token: "actual".into(),
            text: Some("secret".into()),
            error: None,
        })
        .expect("serialize response");

        assert_eq!(
            decode_response(&payload, "expected").unwrap_err(),
            "invalid transcription response token"
        );
    }

    #[test]
    fn remote_error_is_returned_instead_of_text() {
        let payload = serde_json::to_vec(&WaitResponse {
            token: "request".into(),
            text: None,
            error: Some("recording was cancelled".into()),
        })
        .expect("serialize response");

        assert_eq!(
            decode_response(&payload, "request").unwrap_err(),
            "recording was cancelled"
        );
    }
}
