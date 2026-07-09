mod certs;

pub use certs::{get_digital_certificate, save_digital_certificate};

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
        if gtk::is_initialized_main_thread() {
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

        Ok(conn)
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

// Accessor helpers
pub fn get_user_details(conn: &Connection) -> Option<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT full_name, national_id FROM users LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let full_name: String = row.get(0).ok()?;
        let national_id: String = row.get(1).ok()?;
        Some((full_name, national_id))
    } else {
        None
    }
}

pub fn save_user_details(
    conn: &Connection,
    full_name: &str,
    national_id: &str,
) -> Result<(), String> {
    conn.execute("DELETE FROM users", []).ok();
    conn.execute(
        "INSERT INTO users (full_name, national_id) VALUES (?1, ?2)",
        [full_name, national_id],
    )
    .map_err(|e| format!("Failed to save user details: {}", e))?;
    Ok(())
}

pub fn get_smtp_config(conn: &Connection) -> Option<SmtpConfig> {
    let mut stmt = conn
        .prepare("SELECT smtp_server, smtp_port, smtp_user, smtp_pass FROM email_config LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let server: String = row.get(0).ok()?;
        let port: i32 = row.get(1).ok()?;
        let user: String = row.get(2).ok()?;
        let pass: String = row.get(3).ok()?;
        if server.is_empty() {
            None
        } else {
            Some(SmtpConfig {
                server,
                port: port as u16,
                user,
                pass,
            })
        }
    } else {
        None
    }
}

pub fn save_smtp_config(conn: &Connection, config: &SmtpConfig) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id FROM email_config LIMIT 1")
        .map_err(|e| e.to_string())?;
    let exists = stmt.exists([]).map_err(|e| e.to_string())?;
    if exists {
        conn.execute(
            "UPDATE email_config SET smtp_server = ?1, smtp_port = ?2, smtp_user = ?3, smtp_pass = ?4",
            rusqlite::params![config.server, config.port as i32, config.user, config.pass],
        ).map_err(|e| format!("Failed to update SMTP config: {}", e))?;
    } else {
        conn.execute(
            "INSERT INTO email_config (smtp_server, smtp_port, smtp_user, smtp_pass, pop_server, pop_port, pop_user, pop_pass)
             VALUES (?1, ?2, ?3, ?4, '', 995, '', '')",
            rusqlite::params![config.server, config.port as i32, config.user, config.pass],
        ).map_err(|e| format!("Failed to save SMTP config: {}", e))?;
    }
    Ok(())
}

pub fn get_pop_config(conn: &Connection) -> Option<PopConfig> {
    let mut stmt = conn
        .prepare("SELECT pop_server, pop_port, pop_user, pop_pass FROM email_config LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let server: String = row.get(0).ok()?;
        let port: i32 = row.get(1).ok()?;
        let user: String = row.get(2).ok()?;
        let pass: String = row.get(3).ok()?;
        if server.is_empty() {
            None
        } else {
            Some(PopConfig {
                server,
                port: port as u16,
                user,
                pass,
            })
        }
    } else {
        None
    }
}

pub fn save_pop_config(conn: &Connection, config: &PopConfig) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id FROM email_config LIMIT 1")
        .map_err(|e| e.to_string())?;
    let exists = stmt.exists([]).map_err(|e| e.to_string())?;
    if exists {
        conn.execute(
            "UPDATE email_config SET pop_server = ?1, pop_port = ?2, pop_user = ?3, pop_pass = ?4",
            rusqlite::params![config.server, config.port as i32, config.user, config.pass],
        )
        .map_err(|e| format!("Failed to update POP config: {}", e))?;
    } else {
        conn.execute(
            "INSERT INTO email_config (smtp_server, smtp_port, smtp_user, smtp_pass, pop_server, pop_port, pop_user, pop_pass)
             VALUES ('', 587, '', '', ?1, ?2, ?3, ?4)",
            rusqlite::params![config.server, config.port as i32, config.user, config.pass],
        ).map_err(|e| format!("Failed to save POP config: {}", e))?;
    }
    Ok(())
}
pub fn get_credentials_for_domain(
    conn: &Connection,
    domain: &str,
) -> Option<(String, String, String)> {
    let mut stmt = conn
        .prepare("SELECT username, password, email FROM passwords WHERE domain = ?1 OR ?1 LIKE '%' || domain OR domain LIKE '%' || ?1 LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([domain]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let username: String = row.get(0).ok()?;
        let password: String = row.get(1).ok()?;
        let email: String = row.get(2).ok()?;
        Some((username, password, email))
    } else {
        None
    }
}

pub fn save_credentials_for_domain(
    conn: &Connection,
    domain: &str,
    username: &str,
    email: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id FROM passwords WHERE domain = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let exists = stmt.exists([domain]).map_err(|e| e.to_string())?;
    if exists {
        conn.execute(
            "UPDATE passwords SET username = ?2, email = ?3 WHERE domain = ?1",
            [domain, username, email],
        )
        .map_err(|e| format!("Failed to update credentials: {}", e))?;
    } else {
        conn.execute(
            "INSERT INTO passwords (domain, username, password, email) VALUES (?1, ?2, '', ?3)",
            [domain, username, email],
        )
        .map_err(|e| format!("Failed to insert credentials: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
