use std::fs;
use std::path::PathBuf;

use super::certs::delete_digital_certificate;
use super::{
    get_credentials_for_domain, get_digital_certificate, get_pop_config, get_smtp_config,
    get_user_details, save_credentials_for_domain, save_digital_certificate, save_pop_config,
    save_smtp_config, save_user_details, PopConfig, SecureDbManager, SmtpConfig,
};

// lgtm[rust/hardcoded-credentials] -- test fixture constants, not real credentials
#[test]
fn test_db_manager_and_config_save_load() {
    let mut manager = SecureDbManager {
        enc_path: PathBuf::from("test_userdata.enc"),
        temp_path: None,
        key: [0u8; 32],
        salt: [0u8; 16],
    };
    // Clean up any stale test file
    let _ = fs::remove_file(&manager.enc_path);

    let conn = manager.open_connection().unwrap();

    // Verify initially empty
    assert!(get_user_details(&conn).is_none());
    assert!(get_smtp_config(&conn).is_none());
    assert!(get_pop_config(&conn).is_none());

    // Save & Load user details
    save_user_details(&conn, "Juanita Banana", "ID12345").unwrap();
    let user = get_user_details(&conn).unwrap();
    assert_eq!(user.0, "Juanita Banana");
    assert_eq!(user.1, "ID12345");

    // Save & Load SMTP Config
    let smtp = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        user: "user@example.com".to_string(),
        pass: "secret".to_string(),
    };
    save_smtp_config(&conn, &smtp).unwrap();
    let smtp_loaded = get_smtp_config(&conn).unwrap();
    assert_eq!(smtp_loaded.server, "smtp.example.com");
    assert_eq!(smtp_loaded.port, 587);

    // Save & Load POP Config
    let pop = PopConfig {
        server: "pop.example.com".to_string(),
        port: 995,
        user: "popuser@example.com".to_string(),
        pass: "popsecret".to_string(),
    };
    save_pop_config(&conn, &pop).unwrap();
    let pop_loaded = get_pop_config(&conn).unwrap();
    assert_eq!(pop_loaded.server, "pop.example.com");
    assert_eq!(pop_loaded.port, 995);

    // Ensure SMTP config wasn't overwritten by POP config save
    let smtp_loaded_again = get_smtp_config(&conn).unwrap();
    assert_eq!(smtp_loaded_again.server, "smtp.example.com");

    // Save & Load Domain Credentials
    assert!(get_credentials_for_domain(&conn, "google.com").is_none());
    save_credentials_for_domain(&conn, "google.com", "my_user", "my_email@gmail.com").unwrap();
    let creds = get_credentials_for_domain(&conn, "google.com").unwrap();
    assert_eq!(creds.0, "my_user");
    assert_eq!(creds.1, ""); // password is blank
    assert_eq!(creds.2, "my_email@gmail.com");

    // Save & Load Digital Certificates
    assert!(get_digital_certificate(&conn).is_none());
    save_digital_certificate(&conn, "my_fnmt.p12", b"fake_p12_bytes", "cert_secret").unwrap();
    let cert = get_digital_certificate(&conn).unwrap();
    assert_eq!(cert.0, "my_fnmt.p12");
    assert_eq!(cert.1, b"fake_p12_bytes");
    assert_eq!(cert.2, "cert_secret");
    delete_digital_certificate(&conn).unwrap();
    assert!(get_digital_certificate(&conn).is_none());

    manager.save_and_close(conn).unwrap();
    assert!(manager.enc_path.exists());
    let _ = fs::remove_file(&manager.enc_path);
}
