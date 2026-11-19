//! Remote control wire format: the pairing QR payload and the Noise transport.
//! Matches `packages/coding-agent/src/remote-control/protocol.ts` exactly so the
//! Android app can pair with the desktop daemon.

use base64::Engine;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const WEBSOCKET_SUBPROTOCOL: &str = "pibolt-rc.1";
pub const WEBSOCKET_PATH: &str = "/rc";
pub const NOISE_PROLOGUE: &[u8] = b"pibolt-rc/1";
pub const PAIRING_URI_PREFIX: &str = "pibolt://pair#";

/// Data bytes per chunk; with the flag byte this stays under the Noise plaintext limit.
pub const CHUNK_DATA_SIZE: usize = 65_000;
/// The largest message a peer may send, reassembled.
pub const MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

pub const FLAG_FINAL: u8 = 0x00;
pub const FLAG_MORE: u8 = 0x01;

pub const DEFAULT_PAIRING_TTL_MS: u64 = 10 * 60_000;

/// Pairing payload, base64url-encoded in the `pibolt://pair#` URI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairingPayload {
    pub v: u32,
    /// Host display name.
    pub n: String,
    /// Host static public key, base64url.
    pub k: String,
    /// One-time pairing secret, base64url.
    pub s: String,
    /// WebSocket endpoints in preference order.
    pub e: Vec<String>,
    /// Expiry of the secret, Unix seconds.
    pub x: u64,
}

pub fn encode_pairing_uri(payload: &PairingPayload) -> String {
    let json = serde_json::to_string(payload).unwrap();
    format!("{}{}", PAIRING_URI_PREFIX, base64_url_encode(json.as_bytes()))
}

pub fn decode_pairing_uri(uri: &str) -> anyhow::Result<PairingPayload> {
    if !uri.starts_with(PAIRING_URI_PREFIX) {
        anyhow::bail!("Not a Rapid Pi pairing code");
    }
    let payload_bytes = base64_url_decode(&uri[PAIRING_URI_PREFIX.len()..])?;
    let payload: PairingPayload = serde_json::from_slice(&payload_bytes)?;
    if payload.v != PROTOCOL_VERSION {
        anyhow::bail!("Unsupported pairing code version {}", payload.v);
    }
    Ok(payload)
}

/// A code that carries the host's current addresses but pairs nothing (desktop status):
/// its secret is all zeros and it expired at the epoch.
pub fn encode_reconnect_uri(
    host_name: &str,
    host_public_key: &[u8],
    endpoints: &[String],
) -> String {
    let payload = PairingPayload {
        v: PROTOCOL_VERSION,
        n: host_name.to_string(),
        k: base64_url_encode(host_public_key),
        s: base64_url_encode(&[0u8; 32]),
        e: endpoints.to_vec(),
        x: 0,
    };
    encode_pairing_uri(&payload)
}

pub fn base64_url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn base64_url_decode(text: &str) -> anyhow::Result<Vec<u8>> {
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(text)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_pairing_uri_roundtrip() {
        let payload = PairingPayload {
            v: PROTOCOL_VERSION,
            n: "Rapid Pi Desktop".to_string(),
            k: base64_url_encode(&[1u8; 32]),
            s: base64_url_encode(&[2u8; 32]),
            e: vec!["ws://127.0.0.1:48123/rc".to_string()],
            x: 9999999999,
        };
        let uri = encode_pairing_uri(&payload);
        let decoded = decode_pairing_uri(&uri).unwrap();
        assert_eq!(decoded.v, payload.v);
        assert_eq!(decoded.n, payload.n);
        assert_eq!(decoded.k, payload.k);
        assert_eq!(decoded.s, payload.s);
        assert_eq!(decoded.e, payload.e);
        assert_eq!(decoded.x, payload.x);
    }

    #[test]
    fn reject_wrong_prefix() {
        let result = decode_pairing_uri("https://example.com/");
        assert!(result.is_err());
    }
}
