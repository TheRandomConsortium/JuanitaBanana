use gtk::prelude::*;
use gtk::{ApplicationWindow, Dialog, DialogFlags, Entry, Label, ResponseType, Spinner};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{WebView, WebViewExt};

use super::wizard;
use crate::browsing::gui_plugin::GuiPlugin;
use crate::unsubscribe::db::{self, SecureDbManager};

fn decrypt_db_with_spinner(
    parent: &ApplicationWindow,
    pass: &str,
) -> Result<SecureDbManager, String> {
    let dialog = Dialog::with_buttons(
        Some("Decrypting Database"),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[],
    );
    let content = dialog.content_area();
    content.set_margin_start(15);
    content.set_margin_end(15);
    content.set_margin_top(15);
    content.set_margin_bottom(15);
    content.set_spacing(10);

    let label = Label::new(Some("Deriving key using Argon2id...\nPlease wait."));
    content.pack_start(&label, false, false, 0);

    let spinner = Spinner::new();
    spinner.start();
    content.pack_start(&spinner, true, true, 0);

    dialog.show_all();

    while gtk::events_pending() {
        gtk::main_iteration();
    }

    let res = SecureDbManager::new_responsive(pass);

    unsafe {
        dialog.destroy();
    }

    res
}

pub struct AggressiveUnsubscribePlugin;

impl GuiPlugin for AggressiveUnsubscribePlugin {
    fn setup(
        &self,
        window: &ApplicationWindow,
        webview: &WebView,
        _ad_engine: &crate::ad_intoxication::AdIntoxicationEngine,
    ) {
        let unsub_action = gtk::gio::SimpleAction::new("aggressive-unsubscribe", None);
        let window_act = window.clone();
        let webview_act = webview.clone();

        unsub_action.connect_activate(move |_, _| {
            show_unsubscribe_wizard_flow(&window_act, &webview_act);
        });
        window.add_action(&unsub_action);

        let unsub_action_menu = unsub_action.clone();
        webview.connect_context_menu(move |_wv, menu, _event, _hit_test| {
            use webkit2gtk::ContextMenuExt;
            let item = webkit2gtk::ContextMenuItem::from_gaction(
                &unsub_action_menu,
                "Aggressive Unsubscribe 🍌",
                None,
            );
            menu.append(&item);
            false
        });
    }
}

#[allow(unused_assignments)]
pub fn show_unsubscribe_wizard_flow(window: &ApplicationWindow, webview: &WebView) {
    let mut db_manager = None;
    let mut db_conn = None;

    if !SecureDbManager::exists() {
        let dialog = Dialog::with_buttons(
            Some("Opt-in Secure Database"),
            Some(window),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            &[
                ("Enable", ResponseType::Accept),
                ("Cancel", ResponseType::Cancel),
            ],
        );
        let content = dialog.content_area();
        content.set_margin_start(15);
        content.set_margin_end(15);
        content.set_margin_top(15);
        content.set_margin_bottom(15);
        content.set_spacing(10);

        let label = Label::new(Some("Would you like to enable the Secure Database?\n\nThis will store your personal details (necessary for GDPR erasure notices) and your SMTP/POP credentials fully encrypted on your local machine using Argon2id and XChaCha20-Poly1305. It also functions as a Password Manager."));
        label.set_line_wrap(true);
        content.pack_start(&label, false, false, 0);
        dialog.show_all();

        let resp = dialog.run();
        unsafe {
            dialog.destroy();
        }
        if resp != ResponseType::Accept {
            return;
        }

        let pass_dialog = Dialog::with_buttons(
            Some("Create Master Password"),
            Some(window),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            &[
                ("Save", ResponseType::Accept),
                ("Cancel", ResponseType::Cancel),
            ],
        );
        let pass_content = pass_dialog.content_area();
        pass_content.set_margin_start(15);
        pass_content.set_margin_end(15);
        pass_content.set_margin_top(15);
        pass_content.set_margin_bottom(15);
        pass_content.set_spacing(10);

        let label = Label::new(Some("Create a Master Password for your Secure Database:"));
        pass_content.pack_start(&label, false, false, 0);

        let entry = Entry::new();
        entry.set_visibility(false);
        pass_content.pack_start(&entry, false, false, 0);
        pass_dialog.show_all();

        if pass_dialog.run() == ResponseType::Accept {
            let pass = entry.text().to_string();
            unsafe {
                pass_dialog.destroy();
            }
            if pass.is_empty() {
                return;
            }
            match decrypt_db_with_spinner(window, &pass) {
                Ok(mut mgr) => match mgr.open_connection() {
                    Ok(conn) => {
                        db_manager = Some(mgr);
                        db_conn = Some(conn);
                    }
                    Err(e) => {
                        show_error_dialog(window, &format!("Failed to open DB: {}", e));
                        return;
                    }
                },
                Err(e) => {
                    show_error_dialog(window, &format!("Failed to initialize DB: {}", e));
                    return;
                }
            }
        } else {
            unsafe {
                pass_dialog.destroy();
            }
            return;
        }
    } else {
        let mut attempts = 0;
        loop {
            if attempts >= 3 {
                show_error_dialog(window, "Too many failed attempts.");
                return;
            }
            let pass_dialog = Dialog::with_buttons(
                Some("Unlock Secure Database"),
                Some(window),
                DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
                &[
                    ("Decrypt", ResponseType::Accept),
                    ("Cancel", ResponseType::Cancel),
                ],
            );
            let pass_content = pass_dialog.content_area();
            pass_content.set_margin_start(15);
            pass_content.set_margin_end(15);
            pass_content.set_margin_top(15);
            pass_content.set_margin_bottom(15);
            pass_content.set_spacing(10);

            let label = Label::new(Some("Enter Master Password:"));
            pass_content.pack_start(&label, false, false, 0);

            let entry = Entry::new();
            entry.set_visibility(false);
            pass_content.pack_start(&entry, false, false, 0);
            pass_dialog.show_all();

            if pass_dialog.run() == ResponseType::Accept {
                let pass = entry.text().to_string();
                unsafe {
                    pass_dialog.destroy();
                }
                match decrypt_db_with_spinner(window, &pass) {
                    Ok(mut mgr) => match mgr.open_connection() {
                        Ok(conn) => {
                            db_manager = Some(mgr);
                            db_conn = Some(conn);
                            break;
                        }
                        Err(_) => {
                            attempts += 1;
                            show_error_dialog(window, "Invalid password. Please try again.");
                        }
                    },
                    Err(e) => {
                        show_error_dialog(window, &format!("Failed to initialize DB: {}", e));
                        return;
                    }
                }
            } else {
                unsafe {
                    pass_dialog.destroy();
                }
                return;
            }
        }
    }

    let conn = db_conn.as_ref().unwrap();

    let profile = db::get_user_details(conn);
    let (user_name, user_id) = if let Some((name, id)) = profile {
        (name, id)
    } else {
        let prof_dialog = Dialog::with_buttons(
            Some("Setup GDPR Profile Details"),
            Some(window),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            &[("Save Profile", ResponseType::Accept)],
        );
        let prof_content = prof_dialog.content_area();
        prof_content.set_margin_start(15);
        prof_content.set_margin_end(15);
        prof_content.set_margin_top(15);
        prof_content.set_margin_bottom(15);
        prof_content.set_spacing(10);

        let info = Label::new(Some("To generate legally-binding GDPR erasure requests, we need your full name and national ID or passport number."));
        info.set_line_wrap(true);
        prof_content.pack_start(&info, false, false, 0);

        let name_label = Label::new(Some("Full Name:"));
        name_label.set_xalign(0.0);
        let name_entry = Entry::new();
        prof_content.pack_start(&name_label, false, false, 0);
        prof_content.pack_start(&name_entry, false, false, 0);

        let id_label = Label::new(Some("National ID / Passport:"));
        id_label.set_xalign(0.0);
        let id_entry = Entry::new();
        prof_content.pack_start(&id_label, false, false, 0);
        prof_content.pack_start(&id_entry, false, false, 0);

        prof_dialog.show_all();
        let mut n = String::new();
        let mut i = String::new();
        loop {
            if prof_dialog.run() == ResponseType::Accept {
                let name = name_entry.text().to_string();
                let id = id_entry.text().to_string();
                if !name.is_empty() && !id.is_empty() {
                    let _ = db::save_user_details(conn, &name, &id);
                    n = name;
                    i = id;
                    break;
                } else {
                    show_error_dialog(&prof_dialog, "All fields are required.");
                }
            }
        }
        unsafe {
            prof_dialog.destroy();
        }
        (n, i)
    };

    let smtp_config = db::get_smtp_config(conn);
    let pop_config = db::get_pop_config(conn);

    let shared_conn = Rc::new(RefCell::new(db_conn));
    let shared_manager = Rc::new(RefCell::new(db_manager));

    wizard::show_unsubscribe_wizard(
        window,
        webview,
        shared_conn,
        shared_manager,
        user_name,
        user_id,
        smtp_config,
        pop_config,
    );
}

pub fn show_error_dialog<P: IsA<gtk::Window>>(parent: &P, message: &str) {
    let dialog = Dialog::with_buttons(
        Some("Error"),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[("OK", ResponseType::Ok)],
    );
    let content = dialog.content_area();
    content.set_margin_start(15);
    content.set_margin_end(15);
    content.set_margin_top(15);
    content.set_margin_bottom(15);
    let label = Label::new(Some(message));
    content.pack_start(&label, true, true, 0);
    dialog.show_all();
    dialog.run();
    unsafe {
        dialog.destroy();
    }
}

pub fn show_info_dialog<P: IsA<gtk::Window>>(parent: &P, title: &str, message: &str) {
    let dialog = Dialog::with_buttons(
        Some(title),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[("OK", ResponseType::Ok)],
    );
    let content = dialog.content_area();
    content.set_margin_start(15);
    content.set_margin_end(15);
    content.set_margin_top(15);
    content.set_margin_bottom(15);
    let label = Label::new(Some(message));
    label.set_line_wrap(true);
    content.pack_start(&label, true, true, 0);
    dialog.show_all();
    dialog.run();
    unsafe {
        dialog.destroy();
    }
}
