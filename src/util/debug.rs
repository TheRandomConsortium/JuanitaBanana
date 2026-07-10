use lazy_static::lazy_static;

lazy_static! {
    /// Set by passing `--debug` on the command line.
    /// When false (the default) all `[DEBUG]` output is suppressed.
    pub static ref DEBUG_ALL: bool =
        std::env::args().any(|a| a == "--debug");
}

pub fn redact_uri(uri: &str) -> String {
    let Some((base, query)) = uri.split_once('?') else {
        return uri.to_string();
    };
    let redacted: Vec<String> = query
        .split('&')
        .map(|pair| {
            let key = pair.split('=').next().unwrap_or("").to_ascii_lowercase();
            if key == "session" || key.contains("pass") || key.contains("password") {
                format!("{}=<redacted>", pair.split('=').next().unwrap_or(""))
            } else {
                pair.to_string()
            }
        })
        .collect();
    format!("{}?{}", base, redacted.join("&"))
}

#[macro_export]
macro_rules! debug_log {
    // 1. Variante segura (redacta URLs). Ej: debug_log!(GUI, "URL: {}", url)
    ($sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if *$crate::util::debug::DEBUG_ALL { // Usa un flag global
            println!(
                concat!("[DEBUG ", stringify!($sys), "] ", $fmt)
                $(, $crate::util::debug::redact_uri(&$arg.to_string()))*
            );
        }
    };

    // 2. Variante raw (sin redacción). Ej: debug_log!(raw: DB, "Conectado: {}", status)
    (raw: $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if *$crate::util::debug::DEBUG_ALL {
            println!(
                concat!("[DEBUG ", stringify!($sys), "] ", $fmt)
                $(, $arg)*
            );
        }
    };
}
