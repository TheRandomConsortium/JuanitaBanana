use lazy_static::lazy_static;

// Asignamos pesos numéricos directamente a los identificadores
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Info = 1,
    Debug = 2,
}

lazy_static! {
    pub static ref CURRENT_LEVEL: u8 = {
        match std::env::var("JUANITA_LOG") {
            Ok(val) => match val.to_lowercase().as_str() {
                "error" | "0" => LogLevel::Error as u8,
                "info" | "1" | "2" => LogLevel::Info as u8,
                "debug" | "3" => LogLevel::Debug as u8,
                _ => LogLevel::Info as u8,
            },
            Err(_) => LogLevel::Info as u8,
        }
    };
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
macro_rules! log {
    // Specialized Error variant (always prints to stderr via eprintln!)
    (Error, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if ($crate::util::debug::LogLevel::Error as u8) <= *$crate::util::debug::CURRENT_LEVEL {
            eprintln!(
                concat!("[Error] [", stringify!($sys), "] ", $fmt)
                $(, $crate::util::debug::redact_uri(&$arg.to_string()))*
            );
        }
    };
    (raw: Error, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if ($crate::util::debug::LogLevel::Error as u8) <= *$crate::util::debug::CURRENT_LEVEL {
            eprintln!(
                concat!("[Error] [", stringify!($sys), "] ", $fmt)
                $(, $arg)*
            );
        }
    };

    // Generic safe variant
    ($lvl:ident, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if ($crate::util::debug::LogLevel::$lvl as u8) <= *$crate::util::debug::CURRENT_LEVEL {
            println!(
                concat!("[", stringify!($lvl), "] [", stringify!($sys), "] ", $fmt)
                $(, $crate::util::debug::redact_uri(&$arg.to_string()))*
            );
        }
    };

    // Generic raw variant
    (raw: $lvl:ident, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if ($crate::util::debug::LogLevel::$lvl as u8) <= *$crate::util::debug::CURRENT_LEVEL {
            println!(
                concat!("[", stringify!($lvl), "] [", stringify!($sys), "] ", $fmt)
                $(, $arg)*
            );
        }
    };
}
