use rusqlite::Connection;

use super::{PopConfig, SmtpConfig};

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

pub fn save_full_credentials(
    conn: &Connection,
    domain: &str,
    username: &str,
    password: &str,
    email: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id FROM passwords WHERE domain = ?1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let exists = stmt.exists([domain]).map_err(|e| e.to_string())?;
    if exists {
        conn.execute(
            "UPDATE passwords SET username = ?2, password = ?3, email = ?4 WHERE domain = ?1",
            rusqlite::params![domain, username, password, email],
        )
        .map_err(|e| format!("Failed to update full credentials: {}", e))?;
    } else {
        conn.execute(
            "INSERT INTO passwords (domain, username, password, email) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![domain, username, password, email],
        )
        .map_err(|e| format!("Failed to insert full credentials: {}", e))?;
    }
    Ok(())
}

pub fn list_all_credentials(conn: &Connection) -> Vec<(String, String, String, String)> {
    let mut stmt = match conn
        .prepare("SELECT domain, username, password, email FROM passwords ORDER BY domain")
    {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let rows: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, String>(2).unwrap_or_default(),
                row.get::<_, String>(3).unwrap_or_default(),
            ))
        })
        .map(|mapped| mapped.flatten().collect())
        .unwrap_or_default();
    rows
}

pub fn delete_credentials_for_domain(conn: &Connection, domain: &str) -> Result<(), String> {
    conn.execute("DELETE FROM passwords WHERE domain = ?1", [domain])
        .map_err(|e| format!("Failed to delete credentials: {}", e))?;
    Ok(())
}
