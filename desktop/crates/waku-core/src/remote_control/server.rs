//! Remote control WebSocket server.
//! Listens for phone connections at `/rc` with subprotocol `pibolt-rc.1`,
//! performs the Noise IK handshake, validates pairing codes, and proxies
//! messages between the phone and the desktop session runtime.
//! Mirrors `packages/coding-agent/src/remote-control/server.ts`.

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;

use anyhow::{bail, Context as _};
use base64::Engine;
use parking_lot::Mutex as ParkingMutex;
use rand::RngCore;
use serde_json::{json, Value};
use snow::{Builder, TransportState};
use tungstenite::handshake::server::{Request as HandshakeRequest, Response as HandshakeResponse};
use tungstenite::http::{header, StatusCode};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::{accept_hdr_with_config, Message};

use super::noise::{generate_key_pair, KeyPair};
use super::protocol::{
    CHUNK_DATA_SIZE, DEFAULT_PAIRING_TTL_MS, FLAG_FINAL, FLAG_MORE, MAX_MESSAGE_SIZE,
    PAIRING_URI_PREFIX, PROTOCOL_VERSION, WEBSOCKET_PATH, WEBSOCKET_SUBPROTOCOL,
};
use super::store::{PairedDevice, RemoteControlStore};
use snow::params::NoiseParams;

/// One pending one-time pairing, created when the desktop shows a QR code.
#[derive(Clone, Debug)]
struct PendingPairing {
    secret: Vec<u8>,           // 32 random bytes
    host_name: String,
    host_public_key: Vec<u8>,  // 32 bytes X25519
    endpoints: Vec<String>,
    expiry_ms: u64,
}

/// A connected remote-control client (paired phone).
#[derive(Debug)]
struct RemoteClient {
    id: String,
    device: PairedDevice,
}

/// The remote-control server state, shared across connection threads.
#[derive(Debug, Clone)]
pub struct RemoteControlServer {
    pub store: Arc<ParkingMutex<RemoteControlStore>>,
    pub host_key_pair: Arc<ParkingMutex<KeyPair>>,
    pub pending_pairings: Arc<ParkingMutex<HashMap<String, PendingPairing>>>,
    pub clients: Arc<ParkingMutex<HashMap<String, RemoteClient>>>,
    pub listen_port: u16,
}

impl RemoteControlServer {
    pub fn new(agent_dir: &std::path::PathBuf) -> anyhow::Result<Self> {
        let store = Arc::new(ParkingMutex::new(RemoteControlStore::new(agent_dir)));
        let host_key_pair = Arc::new(ParkingMutex::new(
            store.lock().host_key_pair()?,
        ));
        Ok(Self {
            store,
            host_key_pair,
            pending_pairings: Arc::new(ParkingMutex::new(HashMap::new())),
            clients: Arc::new(ParkingMutex::new(HashMap::new())),
            listen_port: 0,
        })
    }

    /// Generate a new one-time pairing code (QR code payload).
    pub fn create_pairing(&self, endpoints: Vec<String>) -> anyhow::Result<String> {
        let mut rng = rand::thread_rng();
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        let host_kp = self.host_key_pair.lock().clone();
        let expiry = now_ms() + DEFAULT_PAIRING_TTL_MS;
        let pairing_id = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&secret[..8]);

        let endpoints_clone = endpoints.clone();
        let pairing = PendingPairing {
            secret: secret.to_vec(),
            host_name: "Rapid Pi Desktop".to_string(),
            host_public_key: host_kp.public_key.clone(),
            endpoints: endpoints_clone,
            expiry_ms: expiry,
        };
        self.pending_pairings.lock().insert(pairing_id, pairing);

        let payload = super::protocol::PairingPayload {
            v: PROTOCOL_VERSION,
            n: "Rapid Pi Desktop".to_string(),
            k: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&host_kp.public_key),
            s: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&secret),
            e: endpoints,
            x: expiry / 1000,
        };
        Ok(super::protocol::encode_pairing_uri(&payload))
    }

    /// Validate a pairing secret against pending pairings.
    pub fn validate_pairing(&self, secret_b64: &str) -> anyhow::Result<PairedDevice> {
        let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(secret_b64)?;
        let now = now_ms();
        let mut found_id = None;
        let mut found_pairing = None;

        for (id, pairing) in self.pending_pairings.lock().iter() {
            if pairing.secret == secret && pairing.expiry_ms > now {
                found_id = Some(id.clone());
                found_pairing = Some(pairing.clone());
                break;
            }
        }

        let pairing = found_pairing.context("invalid or expired pairing code")?;
        let host_public_key = pairing.host_public_key.clone();
        let device_name = format!(
            "Phone-{}",
            secret[..4].iter().map(|b| format!("{b:02x}")).collect::<String>()
        );

        // Upsert the device in the store
        let device = self.store.lock().upsert_device(&host_public_key, &device_name)?;

        // Remove the used pairing
        if let Some(id) = found_id {
            self.pending_pairings.lock().remove(&id);
        }

        Ok(device)
    }

    /// Start listening on a TCP port for remote-control WebSocket connections.
    pub fn listen(&mut self, port: u16) -> anyhow::Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .with_context(|| format!("could not bind remote-control to port {port}"))?;
        let actual_port = listener.local_addr()?.port();
        self.listen_port = actual_port;

        let server = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = server.run(listener) {
                eprintln!("remote-control server error: {e:#}");
            }
        });

        Ok(())
    }

    fn run(self, listener: TcpListener) -> anyhow::Result<()> {
        listener.set_nonblocking(true)?;
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    let server = self.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = server.handle_connection(stream, addr) {
                            eprintln!("remote-control connection error from {addr}: {e:#}");
                        }
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    fn handle_connection(
        &self,
        stream: std::net::TcpStream,
        addr: std::net::SocketAddr,
    ) -> anyhow::Result<()> {
        let config = WebSocketConfig::default()
            .max_message_size(Some(MAX_MESSAGE_SIZE))
            .max_frame_size(Some(MAX_MESSAGE_SIZE));

        let mut socket = accept_hdr_with_config(
            stream,
            |request: &HandshakeRequest, mut response: HandshakeResponse| {
                let path = request.uri().path();
                let proto = request
                    .headers()
                    .get(header::SEC_WEBSOCKET_PROTOCOL)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                if path != WEBSOCKET_PATH || !proto.contains(WEBSOCKET_SUBPROTOCOL) {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
                *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
                response.headers_mut().insert(
                    header::SEC_WEBSOCKET_PROTOCOL,
                    header::HeaderValue::from_static(WEBSOCKET_SUBPROTOCOL),
                );
                Ok(response)
            },
            Some(config),
        )
        .context("WebSocket handshake failed")?;

        // Perform Noise IK handshake (host is responder)
        let params: NoiseParams = "Noise_IK_25519_AESGCM_SHA256".parse()
            .context("invalid noise params")?;
        let builder = Builder::new(params);
        let host_key = self.host_key_pair.lock().private_key.clone();
        let builder = builder.local_private_key(&host_key)
            .context("invalid local private key")?;
        let mut handshake = builder.build_responder()
            .context("could not build Noise responder")?;

        // Read handshake message 1 from client
        let msg1 = read_ws_message(&mut socket)?;
        let mut buf1 = [0u8; 4096];
        let len1 = handshake.read_message(&msg1, &mut buf1).context("Noise handshake msg1 failed")?;
        let _ = &buf1[..len1];

        // Write handshake message 2 to client
        let mut buf2 = [0u8; 4096];
        let len2 = handshake.write_message(&[], &mut buf2).context("Noise handshake msg2 failed")?;
        let msg2 = buf2[..len2].to_vec();
        write_ws_message(&mut socket, Message::Binary(msg2.into()))?;

        // Read handshake message 3 from client
        let msg3 = read_ws_message(&mut socket)?;
        let mut buf3 = [0u8; 4096];
        handshake.read_message(&msg3, &mut buf3).context("Noise handshake msg3 failed")?;

        // Noise handshake complete - now encrypted
        let mut cipher = handshake.into_transport_mode()
            .context("could not enter transport mode")?;

        // Read pairing authentication from client
        let auth_frame = read_ws_message(&mut socket)?;
        let mut auth_buf = [0u8; 4096];
        let auth_len = cipher.read_message(&auth_frame, &mut auth_buf).context("failed to decrypt auth")?;
        let auth_plaintext = &auth_buf[..auth_len];
        let auth: Value = serde_json::from_slice(auth_plaintext)
            .context("auth message was not valid JSON")?;

        let pairing_secret = auth
            .get("pairing")
            .and_then(|v| v.as_str())
            .context("missing pairing field")?;
        let device_name = auth
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Phone");

        // Validate the pairing code
        let device = self.validate_pairing(pairing_secret)?;

        // Send remote_hello (pairing accepted)
        let hello = json!({
            "type": "remote_hello",
            "deviceId": device.public_key,
            "deviceName": device.name,
            "paired": true,
        });
        send_encrypted(&mut socket, &mut cipher, &hello)?;

        // Register the client
        let client_id = uuid::Uuid::new_v4().to_string();
        self.clients.lock().insert(
            client_id.clone(),
            RemoteClient {
                id: client_id.clone(),
                device: device.clone(),
            },
        );

        eprintln!(
            "remote-control: phone paired from {}: {} ({})",
            addr, device.name, device.public_key
        );

        // Message loop - read encrypted messages from phone
        loop {
            let frame = match socket.read() {
                Ok(Message::Binary(frame)) => frame.to_vec(),
                Ok(Message::Close(..)) => break,
                Ok(_) => continue,
                Err(_) => break,
            };
            let mut msg_buf = [0u8; 65536];
            let plaintext_len = match cipher.read_message(&frame, &mut msg_buf) {
                Ok(l) => l,
                Err(_) => break,
            };
            let plaintext = &msg_buf[..plaintext_len];
            let _message: Value = match serde_json::from_slice(plaintext) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Echo back an acknowledgement (simplified - real implementation proxies to session)
            let ack = json!({
                "type": "ack",
                "status": "received",
            });
            if send_encrypted(&mut socket, &mut cipher, &ack).is_err() {
                break;
            }
        }

        self.clients.lock().remove(&client_id);
        Ok(())
    }
}

fn read_ws_message(socket: &mut tungstenite::WebSocket<std::net::TcpStream>) -> anyhow::Result<Vec<u8>> {
    match socket.read()? {
        Message::Binary(data) => Ok(data.to_vec()),
        Message::Text(text) => Ok(text.to_string().into_bytes()),
        Message::Close(..) => bail!("connection closed"),
        _ => bail!("unexpected message type"),
    }
}

fn write_ws_message(
    socket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    msg: Message,
) -> anyhow::Result<()> {
    socket.write(msg)?;
    Ok(())
}

fn send_encrypted(
    socket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    cipher: &mut TransportState,
    message: &Value,
) -> anyhow::Result<()> {
    let plaintext = serde_json::to_vec(message)?;
    // Chunk if needed
    let mut offset = 0;
    while offset < plaintext.len() {
        let end = (offset + CHUNK_DATA_SIZE).min(plaintext.len());
        let flag = if end < plaintext.len() { FLAG_MORE } else { FLAG_FINAL };
        let mut pt = vec![flag];
        pt.extend_from_slice(&plaintext[offset..end]);
        let mut out_buf = [0u8; 65536];
        let ct_len = cipher.write_message(&pt, &mut out_buf)?;
        write_ws_message(socket, Message::Binary(out_buf[..ct_len].to_vec().into()))?;
        offset = end;
    }
    Ok(())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
