//! Remote control's files in the agent directory: the host's static key
//! (`remote-control/host-key`) and the paired phones
//! (`remote-control/devices.json`), both readable only by the user.
//! Matches `packages/coding-agent/src/remote-control/store.ts`.

use std::fs::{create_dir_all, exists, read, set_permissions, write, Permissions};
use std::path::{Path, PathBuf};

use base64::Engine;
use super::noise::KeyPair;
use super::protocol::base64_url_encode;

fn write_private_file(path: &PathBuf, content: &str) -> anyhow::Result<()> {
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    write(&temp, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        set_permissions(&temp, Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&temp, path)?;
    Ok(())
}

/// Remote control store, holding the host key pair and paired device list.
#[derive(Debug)]
pub struct RemoteControlStore {
    pub dir: PathBuf,
}

impl RemoteControlStore {
    pub fn new(agent_dir: &PathBuf) -> Self {
        Self {
            dir: agent_dir.join("remote-control"),
        }
    }

    fn ensure_dir(&self) -> anyhow::Result<()> {
        if !exists(&self.dir).unwrap_or(false) {
            create_dir_all(&self.dir)?;
        }
        Ok(())
    }

    /// The host's static key pair, made on first use.
    /// File format: `public_key_base64\nprivate_key_base64\n`
    pub fn host_key_pair(&self) -> anyhow::Result<KeyPair> {
        let path = self.dir.join("host-key");
        if exists(&path).unwrap_or(false) {
            let content = read(&path)?;
            let text = String::from_utf8_lossy(&content).into_owned();
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() >= 2 {
                let public_key =
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(lines[0].as_bytes())?;
                let private_key =
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(lines[1].as_bytes())?;
                if public_key.len() == 32 && private_key.len() == 32 {
                    return Ok(KeyPair { public_key, private_key });
                }
            }
        }
        self.ensure_dir()?;
        let key_pair = super::noise::generate_key_pair();
        let content = format!(
            "{}\n{}\n",
            base64_url_encode(&key_pair.public_key),
            base64_url_encode(&key_pair.private_key),
        );
        write_private_file(&path, &content)?;
        Ok(key_pair)
    }

    /// List paired devices.
    pub fn devices(&self) -> anyhow::Result<Vec<PairedDevice>> {
        let path = self.dir.join("devices.json");
        if !exists(&path).unwrap_or(false) {
            return Ok(Vec::new());
        }
        let parsed: serde_json::Value = serde_json::from_slice(&read(&path)?)?;
        let arr = parsed
            .get("devices")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("devices.json missing devices array"))?;
        let mut out = Vec::new();
        for entry in arr {
            out.push(PairedDevice {
                public_key: entry["publicKey"].as_str().unwrap_or("").to_string(),
                name: entry["name"].as_str().unwrap_or("").to_string(),
                paired_at: entry["pairedAt"].as_str().unwrap_or("").to_string(),
                last_seen_at: entry["lastSeenAt"].as_str().map(|s| s.to_string()),
            });
        }
        Ok(out)
    }

    /// Add or refresh a device.
    pub fn upsert_device(
        &self,
        public_key: &[u8],
        name: &str,
    ) -> anyhow::Result<PairedDevice> {
        self.ensure_dir()?;
        let key = base64_url_encode(public_key);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let mut devices = self.devices()?;
        let existing = devices.iter().position(|d| d.public_key == key);
        let device = PairedDevice {
            public_key: key.clone(),
            name: name.to_string(),
            paired_at: existing
                .map(|i| devices[i].paired_at.clone())
                .unwrap_or_else(|| now.clone()),
            last_seen_at: Some(now),
        };
        devices.retain(|d| d.public_key != key);
        devices.push(device.clone());
        let json = serde_json::to_string_pretty(&serde_json::json!({ "devices": devices }))?;
        write_private_file(&self.dir.join("devices.json"), &format!("{json}\n"))?;
        Ok(device)
    }

    /// Remove a device by base64url public key.
    pub fn remove_device(&self, public_key: &str) -> anyhow::Result<bool> {
        let devices = self.devices()?;
        let len_before = devices.len();
        let remaining: Vec<_> = devices.into_iter().filter(|d| d.public_key != public_key).collect();
        if remaining.len() == len_before {
            return Ok(false);
        }
        let json = serde_json::to_string_pretty(&serde_json::json!({ "devices": remaining }))?;
        write_private_file(&self.dir.join("devices.json"), &format!("{json}\n"))?;
        Ok(true)
    }
}

/// A paired phone/device.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PairedDevice {
    pub public_key: String,
    pub name: String,
    pub paired_at: String,
    pub last_seen_at: Option<String>,
}

impl AsRef<Path> for RemoteControlStore {
    fn as_ref(&self) -> &Path {
        &self.dir
    }
}
