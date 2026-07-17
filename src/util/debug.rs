use lazy_static::lazy_static;
use std::collections::HashMap;

// Asignamos pesos numéricos directamente a los identificadores
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

lazy_static! {
    pub static ref CURRENT_LEVEL: u8 = {
        match std::env::var("JUANITA_LOG") {
            Ok(val) => match val.to_lowercase().as_str() {
                "error" | "0" => LogLevel::Error as u8,
                "warn" | "1" => LogLevel::Warn as u8,
                "info" | "2" => LogLevel::Info as u8,
                "debug" | "3" => LogLevel::Debug as u8,
                "trace" | "4" => LogLevel::Trace as u8,
                _ => LogLevel::Info as u8,
            },
            Err(_) => LogLevel::Info as u8,
        }
    };
    pub static ref LOG_OVERRIDES: HashMap<String, u8> = {
        let mut map = HashMap::new();
        if let Ok(val) = std::env::var("JUANITA_LOG_OVERRIDE") {
            let parts: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
            for chunk in parts.chunks_exact(2) {
                let sys = chunk[0].to_lowercase();
                let lvl_str = chunk[1].to_lowercase();
                let lvl = match lvl_str.as_str() {
                    "error" | "0" => LogLevel::Error as u8,
                    "warn" | "1" => LogLevel::Warn as u8,
                    "info" | "2" => LogLevel::Info as u8,
                    "debug" | "3" => LogLevel::Debug as u8,
                    "trace" | "4" => LogLevel::Trace as u8,
                    _ => continue,
                };
                map.insert(sys, lvl);
            }
        }
        map
    };
}

pub fn should_log(sys: &str, level: LogLevel) -> bool {
    let level_u8 = level as u8;
    if let Some(&override_lvl) = LOG_OVERRIDES.get(&sys.to_lowercase()) {
        level_u8 <= override_lvl
    } else {
        level_u8 <= *CURRENT_LEVEL
    }
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
        if $crate::util::debug::should_log(stringify!($sys), $crate::util::debug::LogLevel::Error) {
            eprintln!(
                concat!("[Error] [", stringify!($sys), "] ", $fmt)
                $(, $crate::util::debug::redact_uri(&$arg.to_string()))*
            );
        }
    };
    (raw: Error, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if $crate::util::debug::should_log(stringify!($sys), $crate::util::debug::LogLevel::Error) {
            eprintln!(
                concat!("[Error] [", stringify!($sys), "] ", $fmt)
                $(, $arg)*
            );
        }
    };

    // Generic safe variant
    ($lvl:ident, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if $crate::util::debug::should_log(stringify!($sys), $crate::util::debug::LogLevel::$lvl) {
            println!(
                concat!("[", stringify!($lvl), "] [", stringify!($sys), "] ", $fmt)
                $(, $crate::util::debug::redact_uri(&$arg.to_string()))*
            );
        }
    };

    // Generic raw variant
    (raw: $lvl:ident, $sys:ident, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if $crate::util::debug::should_log(stringify!($sys), $crate::util::debug::LogLevel::$lvl) {
            println!(
                concat!("[", stringify!($lvl), "] [", stringify!($sys), "] ", $fmt)
                $(, $arg)*
            );
        }
    };
}

pub fn console_override_script() -> &'static str {
    r#"
    (function() {
        const intercept = (method) => {
            const orig = console[method];
            if (!orig) return;
            console[method] = function(...args) {
                orig.apply(console, args);
                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.console) {
                    const msg = args.map(a => {
                        if (a === null) return "null";
                        if (a === undefined) return "undefined";
                        if (typeof a === 'object') {
                            try {
                                return JSON.stringify(a);
                            } catch(e) {
                                return String(a);
                            }
                        }
                        return String(a);
                    }).join(' ');
                    window.webkit.messageHandlers.console.postMessage(`[${method.toUpperCase()}] ${msg}`);
                }
            };
        };
        intercept('log');
        intercept('warn');
        intercept('error');
        intercept('info');
        intercept('debug');
    })();
    "#
}
