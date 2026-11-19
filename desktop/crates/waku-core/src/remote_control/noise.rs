//! Noise IK static key pair generation for remote control host identity.

use snow::Builder;

/// Generate a new Noise static key pair (X25519).
pub fn generate_key_pair() -> KeyPair {
    let params: snow::params::NoiseParams =
        "Noise_IK_25519_AESGCM_SHA256".parse().expect("valid noise params");
    let builder = Builder::new(params);
    let keypair = builder.generate_keypair().expect("keypair generation");
    KeyPair {
        private_key: keypair.private,
        public_key: keypair.public,
    }
}

/// A Noise static key pair.
#[derive(Clone, Debug)]
pub struct KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl KeyPair {
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }
}
