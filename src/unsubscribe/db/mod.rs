mod certs;
mod queries;

pub use certs::{get_digital_certificate, save_digital_certificate};
pub use queries::*;

pub type P2pNodeKeyPairBytes = ([u8; 32], [u8; 32]);

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use rand::Rng;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

#[derive(Clone, Debug)]
pub struct PopConfig {
    pub server: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

pub struct SecureDbManager {
    enc_path: PathBuf,
    temp_path: Option<PathBuf>,
    key: [u8; 32],
    salt: [u8; 16],
}

impl SecureDbManager {
    pub fn file_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
            });
        let mut path = base.join("juanita-banana");
        fs::create_dir_all(&path).ok();
        path.push("userdata.enc");
        path
    }

    pub fn exists() -> bool {
        Self::file_path().exists()
    }

    pub fn new_responsive(password: &str) -> Result<Self, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let pass_str = password.to_string();
        std::thread::spawn(move || {
            let res = Self::new(&pass_str);
            let _ = tx.send(res);
        });

        let mut result = None;
        if gtk::is_initialized_main_thread() && gtk::glib::MainContext::default().is_owner() {
            loop {
                match rx.try_recv() {
                    Ok(res) => {
                        result = Some(res);
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        while gtk::events_pending() {
                            gtk::main_iteration();
                        }
                        std::thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
            }
        }

        match result {
            Some(res) => res,
            None => rx
                .recv()
                .map_err(|e| format!("Channel receive error: {}", e))?,
        }
    }

    pub fn new(password: &str) -> Result<Self, String> {
        let enc_path = Self::file_path();
        let mut key = [0u8; 32];

        let salt = if enc_path.exists() {
            let data =
                fs::read(&enc_path).map_err(|e| format!("Failed to read database: {}", e))?;
            if data.len() < 16 {
                return Err("Corrupted database file (too short)".to_string());
            }
            let mut s = [0u8; 16];
            s.copy_from_slice(&data[0..16]);
            s
        } else {
            let mut s = [0u8; 16];
            rand::thread_rng().fill(&mut s);
            s
        };

        // Derive key using Argon2id: 1GB memory, 1 iteration, 1 thread
        let params = Params::new(1024 * 1024, 1, 1, Some(32))
            .map_err(|e| format!("Argon2 params error: {}", e))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        argon
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        Ok(SecureDbManager {
            enc_path,
            temp_path: None,
            key,
            salt,
        })
    }

    pub fn open_connection(&mut self) -> Result<Connection, String> {
        let shm = Path::new("/dev/shm");
        let temp_dir = if shm.exists() && shm.is_dir() {
            shm.to_path_buf()
        } else {
            std::env::temp_dir()
        };

        let temp_name = format!("juanita-banana-db-{}", rand::thread_rng().gen::<u64>());
        let temp_path = temp_dir.join(temp_name);
        self.temp_path = Some(temp_path.clone());

        if self.enc_path.exists() {
            let data =
                fs::read(&self.enc_path).map_err(|e| format!("Failed to read database: {}", e))?;
            if data.len() < 40 {
                return Err("Corrupted database file (too short)".to_string());
            }
            let mut nonce = [0u8; 24];
            nonce.copy_from_slice(&data[16..40]);
            let ciphertext = &data[40..];

            let cipher = XChaCha20Poly1305::new(&self.key.into());
            let decrypted = cipher
                .decrypt(&nonce.into(), ciphertext)
                .map_err(|_| "Decryption failed. Invalid password?".to_string())?;

            fs::write(&temp_path, decrypted)
                .map_err(|e| format!("Failed to write decrypted file: {}", e))?;
        }

        let conn =
            Connection::open(&temp_path).map_err(|e| format!("Failed to open SQLite: {}", e))?;

        // Ensure table schemas exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                full_name TEXT NOT NULL,
                national_id TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Schema error (users): {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS email_config (
                id INTEGER PRIMARY KEY,
                smtp_server TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                smtp_user TEXT NOT NULL,
                smtp_pass TEXT NOT NULL,
                pop_server TEXT NOT NULL DEFAULT '',
                pop_port INTEGER NOT NULL DEFAULT 995,
                pop_user TEXT NOT NULL DEFAULT '',
                pop_pass TEXT NOT NULL DEFAULT ''
            )",
            [],
        )
        .map_err(|e| format!("Schema error (email_config): {}", e))?;

        let _ = conn.execute(
            "ALTER TABLE email_config ADD COLUMN pop_server TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE email_config ADD COLUMN pop_port INTEGER NOT NULL DEFAULT 995",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE email_config ADD COLUMN pop_user TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE email_config ADD COLUMN pop_pass TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE passwords ADD COLUMN email TEXT NOT NULL DEFAULT ''",
            [],
        );

        conn.execute(
            "CREATE TABLE IF NOT EXISTS passwords (
                id INTEGER PRIMARY KEY,
                domain TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT ''
            )",
            [],
        )
        .map_err(|e| format!("Schema error (passwords): {}", e))?;

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS digital_certificates (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                cert_blob BLOB NOT NULL,
                password TEXT NOT NULL
            )",
            [],
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS p2p_node_keys (
                id INTEGER PRIMARY KEY,
                secret_key BLOB NOT NULL,
                public_key BLOB NOT NULL
            )",
            [],
        );

        Ok(conn)
    }

    pub fn get_p2p_node_key(&mut self) -> Result<Option<P2pNodeKeyPairBytes>, String> {
        let conn = self.open_connection()?;
        let res = conn.query_row(
            "SELECT secret_key, public_key FROM p2p_node_keys ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                let sec: Vec<u8> = row.get(0)?;
                let pubk: Vec<u8> = row.get(1)?;
                Ok((sec, pubk))
            },
        );
        let _ = self.save_and_close(conn);
        match res {
            Ok((sec, pubk)) if sec.len() == 32 && pubk.len() == 32 => {
                let mut s = [0u8; 32];
                let mut p = [0u8; 32];
                s.copy_from_slice(&sec);
                p.copy_from_slice(&pubk);
                Ok(Some((s, p)))
            }
            Ok(_) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Query error: {}", e)),
        }
    }

    pub fn save_p2p_node_key(
        &mut self,
        secret_key: &[u8; 32],
        public_key: &[u8; 32],
    ) -> Result<(), String> {
        let conn = self.open_connection()?;
        conn.execute(
            "INSERT INTO p2p_node_keys (secret_key, public_key) VALUES (?, ?)",
            rusqlite::params![secret_key.as_ref(), public_key.as_ref()],
        )
        .map_err(|e| format!("Save key error: {}", e))?;
        self.save_and_close(conn)
    }

    pub fn save_and_close(&mut self, conn: Connection) -> Result<(), String> {
        let temp_path = match &self.temp_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        // Drop the connection to release SQLite lock
        drop(conn);

        if temp_path.exists() {
            let plaintext =
                fs::read(&temp_path).map_err(|e| format!("Failed to read temp file: {}", e))?;

            let mut nonce = [0u8; 24];
            rand::thread_rng().fill(&mut nonce);

            let cipher = XChaCha20Poly1305::new(&self.key.into());
            let ciphertext = cipher
                .encrypt(&nonce.into(), plaintext.as_slice())
                .map_err(|e| format!("Encryption failed: {}", e))?;

            // File format: [16 bytes salt][24 bytes nonce][ciphertext]
            let mut final_data = Vec::with_capacity(16 + 24 + ciphertext.len());
            final_data.extend_from_slice(&self.salt);
            final_data.extend_from_slice(&nonce);
            final_data.extend_from_slice(&ciphertext);

            fs::write(&self.enc_path, final_data)
                .map_err(|e| format!("Failed to write encrypted database: {}", e))?;

            // Cleanup the decrypted temp file
            let _ = fs::remove_file(&temp_path);
        }

        self.temp_path = None;
        Ok(())
    }
}

impl Drop for SecureDbManager {
    fn drop(&mut self) {
        if let Some(p) = &self.temp_path {
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
    }
}

#[cfg(test)]
mod tests;
