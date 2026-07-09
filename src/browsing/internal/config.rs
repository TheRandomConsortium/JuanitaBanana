use super::{InternalPage, PageContext};
use crate::util::config::AppConfig;
use webkit2gtk::{UserContentManagerExt, WebViewExt};

fn get_query_param(uri: &str, key: &str) -> Option<String> {
    let parts: Vec<&str> = uri.split('?').collect();
    if parts.len() < 2 {
        return None;
    }
    let query = parts[1];
    for pair in query.split('&') {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 && kv[0] == key {
            if let Ok(decoded) = urlencoding::decode(kv[1]) {
                return Some(decoded.into_owned());
            }
        }
    }
    None
}

pub struct ConfigPage;

impl InternalPage for ConfigPage {
    fn matches_input(&self, input: &str) -> bool {
        input.starts_with("juanita:config") || input.starts_with("juanita://config")
    }

    fn handle_input(&self, input: &str, ctx: &PageContext) {
        ctx.webview.load_uri(input);
    }

    fn matches_policy(&self, uri: &str) -> bool {
        // Match all except the config-page target to prevent loop
        (uri.starts_with("juanita://config") && !uri.starts_with("juanita://config-page"))
            || uri.starts_with("juanita://save-config")
            || uri.starts_with("juanita://save-secure-config")
            || uri.starts_with("juanita://make-default")
    }

    fn ignore_policy(&self, _uri: &str) -> bool {
        true
    }

    fn handle_policy(&self, uri: &str, ctx: &PageContext) -> bool {
        if uri.starts_with("juanita://config") {
            let is_default = crate::util::config::is_default_browser();
            let mut decrypted = None;
            let mut unlock_error = false;

            if let Some(pass) = get_query_param(uri, "unlock_pass") {
                // Try to decrypt the database
                match crate::unsubscribe::db::SecureDbManager::new(&pass) {
                    Ok(mut manager) => match manager.open_connection() {
                        Ok(conn) => {
                            let profile = crate::unsubscribe::db::get_user_details(&conn);
                            let smtp = crate::unsubscribe::db::get_smtp_config(&conn);
                            let pop = crate::unsubscribe::db::get_pop_config(&conn);

                            let name = profile.as_ref().map(|p| p.0.clone()).unwrap_or_default();
                            let id = profile.as_ref().map(|p| p.1.clone()).unwrap_or_default();

                            let smtp_server =
                                smtp.as_ref().map(|s| s.server.clone()).unwrap_or_default();
                            let smtp_port = smtp.as_ref().map(|s| s.port).unwrap_or(587);
                            let smtp_user =
                                smtp.as_ref().map(|s| s.user.clone()).unwrap_or_default();
                            let smtp_pass =
                                smtp.as_ref().map(|s| s.pass.clone()).unwrap_or_default();

                            let pop_server =
                                pop.as_ref().map(|p| p.server.clone()).unwrap_or_default();
                            let pop_port = pop.as_ref().map(|p| p.port).unwrap_or(995);
                            let pop_user = pop.as_ref().map(|p| p.user.clone()).unwrap_or_default();
                            let pop_pass = pop.as_ref().map(|p| p.pass.clone()).unwrap_or_default();

                            decrypted = Some(crate::util::config_html::DecryptedSecureData {
                                master_pass: pass,
                                name,
                                id,
                                smtp_server,
                                smtp_port,
                                smtp_user,
                                smtp_pass,
                                pop_server,
                                pop_port,
                                pop_user,
                                pop_pass,
                            });
                            let _ = manager.save_and_close(conn);
                        }
                        Err(_) => {
                            unlock_error = true;
                        }
                    },
                    Err(_) => {
                        unlock_error = true;
                    }
                }
            } else if get_query_param(uri, "unlock_error").is_some() {
                unlock_error = true;
            }

            let config_html = crate::util::config_html::config_page_html(
                &ctx.config,
                is_default,
                decrypted.as_ref(),
                unlock_error,
            );
            let base_uri = uri.replace("juanita://config", "juanita://config-page");
            ctx.webview.load_html(&config_html, Some(&base_uri));
            return true;
        }

        if uri.starts_with("juanita://save-secure-config") {
            let pass = get_query_param(uri, "pass").unwrap_or_default();
            let name = get_query_param(uri, "name").unwrap_or_default();
            let id = get_query_param(uri, "id").unwrap_or_default();

            let smtp_server = get_query_param(uri, "smtp_server").unwrap_or_default();
            let smtp_port = get_query_param(uri, "smtp_port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(587);
            let smtp_user = get_query_param(uri, "smtp_user").unwrap_or_default();
            let smtp_pass = get_query_param(uri, "smtp_pass").unwrap_or_default();

            let pop_server = get_query_param(uri, "pop_server").unwrap_or_default();
            let pop_port = get_query_param(uri, "pop_port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(995);
            let pop_user = get_query_param(uri, "pop_user").unwrap_or_default();
            let pop_pass = get_query_param(uri, "pop_pass").unwrap_or_default();

            let mut success = false;
            if !pass.is_empty() {
                if let Ok(mut manager) = crate::unsubscribe::db::SecureDbManager::new(&pass) {
                    if let Ok(conn) = manager.open_connection() {
                        let _ = crate::unsubscribe::db::save_user_details(&conn, &name, &id);

                        let smtp = crate::unsubscribe::db::SmtpConfig {
                            server: smtp_server,
                            port: smtp_port,
                            user: smtp_user,
                            pass: smtp_pass,
                        };
                        let _ = crate::unsubscribe::db::save_smtp_config(&conn, &smtp);

                        let pop = crate::unsubscribe::db::PopConfig {
                            server: pop_server,
                            port: pop_port,
                            user: pop_user,
                            pass: pop_pass,
                        };
                        let _ = crate::unsubscribe::db::save_pop_config(&conn, &pop);

                        let _ = manager.save_and_close(conn);
                        success = true;
                    }
                }
            }

            let redirect_uri = if success {
                format!(
                    "juanita://config?saved_secure=true&unlock_pass={}",
                    urlencoding::encode(&pass)
                )
            } else {
                "juanita://config?unlock_error=true".to_string()
            };
            ctx.webview.load_uri(&redirect_uri);
            return true;
        }

        if let Some(data_str) = uri.strip_prefix("juanita://save-config?data=") {
            if let Ok(decoded) = urlencoding::decode(data_str) {
                if let Ok(new_config) = serde_json::from_str::<AppConfig>(&decoded) {
                    new_config.save();
                    println!("[CONFIG] Configuration saved successfully. Reloading scripts.");

                    if let Some(ucm) = ctx.webview.user_content_manager() {
                        ucm.remove_all_scripts();

                        let fp_script = webkit2gtk::UserScript::new(
                            crate::fingerprint::spoof::anti_fingerprint_script(),
                            webkit2gtk::UserContentInjectedFrames::AllFrames,
                            webkit2gtk::UserScriptInjectionTime::Start,
                            &[],
                            &[],
                        );
                        ucm.add_script(&fp_script);

                        let ad_script = webkit2gtk::UserScript::new(
                            &crate::ad_intoxication::ad_intoxication_script(&new_config),
                            webkit2gtk::UserContentInjectedFrames::AllFrames,
                            webkit2gtk::UserScriptInjectionTime::Start,
                            &[],
                            &[],
                        );
                        ucm.add_script(&ad_script);

                        let toxic_script = webkit2gtk::UserScript::new(
                            &crate::util::ban::toxic_warning_script(&new_config),
                            webkit2gtk::UserContentInjectedFrames::TopFrame,
                            webkit2gtk::UserScriptInjectionTime::Start,
                            &[],
                            &[],
                        );
                        ucm.add_script(&toxic_script);
                    }
                }
            }
            ctx.webview.load_uri("juanita://config?saved=true");
            return true;
        }

        if uri.starts_with("juanita://make-default") {
            let exe_path = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("juanita-banana"));
            let is_system_install = exe_path.starts_with("/usr/");

            let desktop_filename = if is_system_install {
                "juanita-banana.desktop".to_string()
            } else {
                let base = std::env::var("XDG_DATA_HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                            .join(".local/share")
                    });
                let apps_dir = base.join("applications");
                std::fs::create_dir_all(&apps_dir).ok();

                let desktop_path = apps_dir.join("juanita-banana-local.desktop");
                let desktop_content = format!(
                    "[Desktop Entry]\nVersion=1.0\nName=Juanita Banana (Local)\nGenericName=Web Browser\nComment=Weaponized Privacy Browser\nExec={} %U\nTerminal=false\nX-MultipleArgs=false\nType=Application\nIcon=web-browser\nCategories=Network;WebBrowser;\nMimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/juanita;\nStartupNotify=true",
                    exe_path.display()
                );
                std::fs::write(&desktop_path, desktop_content).ok();
                "juanita-banana-local.desktop".to_string()
            };

            std::process::Command::new("xdg-settings")
                .arg("set")
                .arg("default-web-browser")
                .arg(&desktop_filename)
                .spawn()
                .ok();

            println!("[CONFIG] Set as default browser!");
            ctx.webview.load_uri("juanita://config");
            return true;
        }

        false
    }
}
