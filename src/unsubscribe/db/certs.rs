//! Digital certificate storage helpers.
//!
//! Persists PKCS#12 certificate blobs inside the encrypted SecureDb so that
//! they can be retrieved at report-generation time to produce a CAdES signature.

use rusqlite::Connection;

/// Returns `(name, cert_blob, password)` for the single stored certificate, if any.
pub fn get_digital_certificate(conn: &Connection) -> Option<(String, Vec<u8>, String)> {
    let mut stmt = conn
        .prepare("SELECT name, cert_blob, password FROM digital_certificates LIMIT 1")
        .ok()?;
    let mut rows = stmt.query([]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let name: String = row.get(0).ok()?;
        let cert_blob: Vec<u8> = row.get(1).ok()?;
        let password: String = row.get(2).ok()?;
        Some((name, cert_blob, password))
    } else {
        None
    }
}

/// Replaces the stored certificate (only one is kept at a time).
pub fn save_digital_certificate(
    conn: &Connection,
    name: &str,
    cert_blob: &[u8],
    password: &str,
) -> Result<(), String> {
    let _ = conn.execute("DELETE FROM digital_certificates", []);
    conn.execute(
        "INSERT INTO digital_certificates (name, cert_blob, password) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, cert_blob, password],
    )
    .map_err(|e| format!("Failed to save digital certificate: {}", e))?;
    Ok(())
}

/// Removes the stored certificate.
#[allow(dead_code)]
pub fn delete_digital_certificate(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM digital_certificates", [])
        .map_err(|e| format!("Failed to delete digital certificate: {}", e))?;
    Ok(())
}
