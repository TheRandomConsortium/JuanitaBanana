#![allow(clippy::too_many_arguments)]
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText, Dialog, Entry, Label,
    Notebook, Orientation, RadioButton, Spinner,
};
use rusqlite::Connection;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::report_action::handle_report_generation;
use super::{show_error_dialog, show_info_dialog};
use crate::unsubscribe::crawler;
use crate::unsubscribe::db::SmtpConfig;
use crate::unsubscribe::email;
use crate::unsubscribe::registry::UnsubscribeRegistry;

pub fn handle_next_click(
    step: u32,
    window_next: &ApplicationWindow,
    wizard_next: &Dialog,
    notebook_next: &Notebook,
    btn_back_next: &Button,
    btn_next_next: &Button,
    rb_unsub_next: &RadioButton,
    rb_report_next: &RadioButton,
    rb_curr_next: &RadioButton,
    rb_manual_next: &RadioButton,
    rb_search_next: &RadioButton,
    entry_manual_next: &Entry,
    entry_search_next: &Entry,
    combo_ddg_next: &ComboBoxText,
    combo_notified_next: &ComboBoxText,
    dpo_email_entry_next: &Entry,
    entry_user_email_next: &Entry,
    entry_user_username_next: &Entry,
    spinner_next: &Spinner,
    emails_box_next: &GtkBox,
    text_preview_next: &gtk::TextView,
    current_step_next: &Rc<Cell<u32>>,
    selected_domain_next: &Rc<RefCell<String>>,
    selected_emails_next: &Rc<RefCell<Vec<String>>>,
    active_smtp_next: &Rc<RefCell<Option<SmtpConfig>>>,
    checkbuttons_list_next: &Rc<RefCell<Vec<CheckButton>>>,
    shared_conn_next: &Rc<RefCell<Option<Connection>>>,
    user_name_next: &str,
    user_id_next: &str,
    current_domain_next: &str,
    registry_next: &Rc<RefCell<UnsubscribeRegistry>>,
) {
    match step {
        0 => {
            if rb_unsub_next.is_active() {
                notebook_next.set_current_page(Some(1));
                current_step_next.set(1);
                btn_back_next.set_sensitive(true);
            } else if rb_report_next.is_active() {
                notebook_next.set_current_page(Some(5));
                current_step_next.set(5);
                btn_back_next.set_sensitive(true);
                btn_next_next.set_label("Finish");
            }
        }
        1 => {
            let domain = if rb_curr_next.is_active() {
                current_domain_next.to_string()
            } else if rb_manual_next.is_active() {
                entry_manual_next.text().to_string()
            } else if rb_search_next.is_active() {
                let query = entry_search_next.text().to_string();
                if query.is_empty() {
                    return;
                }
                btn_next_next.set_sensitive(false);

                let (tx, rx) = std::sync::mpsc::channel::<Vec<String>>();

                let combo_ddg_c = combo_ddg_next.clone();
                let current_step_c = current_step_next.clone();
                let notebook_c = notebook_next.clone();
                let btn_next_c = btn_next_next.clone();
                let btn_back_c = btn_back_next.clone();

                gtk::glib::idle_add_local(move || {
                    if let Ok(domains) = rx.try_recv() {
                        combo_ddg_c.remove_all();
                        for d in &domains {
                            combo_ddg_c.append_text(d);
                        }
                        if !domains.is_empty() {
                            combo_ddg_c.set_active(Some(0));
                            notebook_c.set_current_page(Some(2));
                            current_step_c.set(2);
                            btn_next_c.set_sensitive(true);
                            btn_back_c.set_sensitive(true);
                        } else {
                            btn_next_c.set_sensitive(true);
                        }
                        gtk::glib::ControlFlow::Break
                    } else {
                        gtk::glib::ControlFlow::Continue
                    }
                });

                std::thread::spawn(move || {
                    let domains = crawler::search_ddg_domains(&query);
                    let _ = tx.send(domains);
                });
                return;
            } else {
                String::new()
            };

            if domain.is_empty() {
                return;
            }

            if registry_next.borrow().is_notified(&domain) {
                show_error_dialog(
                    wizard_next,
                    &format!(
                        "Warning: {} was already unsubscribed. Redirecting to Report Reincident page...",
                        domain
                    ),
                );
                notebook_next.set_current_page(Some(5));
                current_step_next.set(5);
                btn_next_next.set_label("Finish");
                let notified_list: Vec<String> = registry_next
                    .borrow()
                    .notified_domains
                    .keys()
                    .cloned()
                    .collect();
                if let Some(pos) = notified_list.iter().position(|x| x == &domain) {
                    combo_notified_next.set_active(Some(pos as u32));
                }
                if let Some(entry) = registry_next.borrow().get_domain(&domain) {
                    if !entry.emails_used.is_empty() {
                        dpo_email_entry_next.set_text(&entry.emails_used[0]);
                    }
                }
                return;
            }

            entry_user_email_next.set_text("");
            entry_user_username_next.set_text("");
            if let Some(conn) = shared_conn_next.borrow().as_ref() {
                if let Some((username, _pass, email_val)) =
                    crate::unsubscribe::db::get_credentials_for_domain(conn, &domain)
                {
                    entry_user_email_next.set_text(&email_val);
                    entry_user_username_next.set_text(&username);
                }
            }

            *selected_domain_next.borrow_mut() = domain.clone();

            notebook_next.set_current_page(Some(3));
            current_step_next.set(3);
            btn_next_next.set_sensitive(false);
            spinner_next.start();

            let (tx, rx) = std::sync::mpsc::channel::<crawler::CrawlResult>();

            let spinner_c = spinner_next.clone();
            let emails_box_c = emails_box_next.clone();
            let btn_next_c = btn_next_next.clone();
            let checkbuttons_list_c = checkbuttons_list_next.clone();

            gtk::glib::idle_add_local(move || {
                if let Ok(res) = rx.try_recv() {
                    spinner_c.stop();
                    emails_box_c.foreach(|w| emails_box_c.remove(w));
                    checkbuttons_list_c.borrow_mut().clear();

                    let label = if res.emails.is_empty() {
                        Label::new(Some(
                            "No contact emails found. Please enter one manually below:",
                        ))
                    } else {
                        Label::new(Some("Select contact emails for unsubscribe notice:"))
                    };
                    emails_box_c.pack_start(&label, false, false, 0);

                    for email in &res.emails {
                        let cb = CheckButton::with_label(email);
                        cb.set_active(true);
                        emails_box_c.pack_start(&cb, false, false, 0);
                        checkbuttons_list_c.borrow_mut().push(cb);
                    }

                    let manual_box = GtkBox::new(Orientation::Horizontal, 5);
                    let manual_entry = Entry::new();
                    manual_entry.set_placeholder_text(Some("custom@email.com"));
                    let add_btn = Button::with_label("Add");

                    let emails_box_add = emails_box_c.clone();
                    let checkbuttons_list_add = checkbuttons_list_c.clone();
                    let manual_entry_c = manual_entry.clone();
                    add_btn.connect_clicked(move |_| {
                        let text = manual_entry_c.text().to_string();
                        if !text.is_empty() && text.contains('@') {
                            let cb = CheckButton::with_label(&text);
                            cb.set_active(true);
                            emails_box_add.pack_start(&cb, false, false, 0);
                            checkbuttons_list_add.borrow_mut().push(cb);
                            emails_box_add.show_all();
                            manual_entry_c.set_text("");
                        }
                    });

                    let manual_entry_widget = manual_entry.clone();
                    manual_box.pack_start(&manual_entry_widget, true, true, 0);
                    manual_box.pack_start(&add_btn, false, false, 0);
                    emails_box_c.pack_start(&manual_box, false, false, 0);

                    emails_box_c.show_all();
                    btn_next_c.set_sensitive(true);
                    gtk::glib::ControlFlow::Break
                } else {
                    gtk::glib::ControlFlow::Continue
                }
            });

            let domain_c = domain;
            std::thread::spawn(move || {
                let res = crawler::crawl_domain(domain_c);
                let _ = tx.send(res);
            });
        }
        2 => {
            if let Some(domain) = combo_ddg_next.active_text() {
                let domain_str = domain.as_str().to_string();

                if registry_next.borrow().is_notified(&domain_str) {
                    show_error_dialog(
                        wizard_next,
                        &format!(
                            "Warning: {} was already unsubscribed. Redirecting to Report Reincident page...",
                            domain_str
                        ),
                    );
                    notebook_next.set_current_page(Some(5));
                    current_step_next.set(5);
                    btn_next_next.set_label("Finish");
                    let notified_list: Vec<String> = registry_next
                        .borrow()
                        .notified_domains
                        .keys()
                        .cloned()
                        .collect();
                    if let Some(pos) = notified_list.iter().position(|x| x == &domain_str) {
                        combo_notified_next.set_active(Some(pos as u32));
                    }
                    if let Some(entry) = registry_next.borrow().get_domain(&domain_str) {
                        if !entry.emails_used.is_empty() {
                            dpo_email_entry_next.set_text(&entry.emails_used[0]);
                        }
                    }
                    return;
                }

                entry_user_email_next.set_text("");
                entry_user_username_next.set_text("");
                if let Some(conn) = shared_conn_next.borrow().as_ref() {
                    if let Some((username, _pass, email_val)) =
                        crate::unsubscribe::db::get_credentials_for_domain(conn, &domain_str)
                    {
                        entry_user_email_next.set_text(&email_val);
                        entry_user_username_next.set_text(&username);
                    }
                }

                *selected_domain_next.borrow_mut() = domain_str.clone();

                notebook_next.set_current_page(Some(3));
                current_step_next.set(3);
                btn_next_next.set_sensitive(false);
                spinner_next.start();

                let (tx, rx) = std::sync::mpsc::channel::<crawler::CrawlResult>();

                let spinner_c = spinner_next.clone();
                let emails_box_c = emails_box_next.clone();
                let btn_next_c = btn_next_next.clone();
                let checkbuttons_list_c = checkbuttons_list_next.clone();

                gtk::glib::idle_add_local(move || {
                    if let Ok(res) = rx.try_recv() {
                        spinner_c.stop();
                        emails_box_c.foreach(|w| emails_box_c.remove(w));
                        checkbuttons_list_c.borrow_mut().clear();

                        let label = if res.emails.is_empty() {
                            Label::new(Some(
                                "No contact emails found. Please enter one manually below:",
                            ))
                        } else {
                            Label::new(Some("Select contact emails for unsubscribe notice:"))
                        };
                        emails_box_c.pack_start(&label, false, false, 0);

                        for email in &res.emails {
                            let cb = CheckButton::with_label(email);
                            cb.set_active(true);
                            emails_box_c.pack_start(&cb, false, false, 0);
                            checkbuttons_list_c.borrow_mut().push(cb);
                        }

                        let manual_box = GtkBox::new(Orientation::Horizontal, 5);
                        let manual_entry = Entry::new();
                        manual_entry.set_placeholder_text(Some("custom@email.com"));
                        let add_btn = Button::with_label("Add");

                        let emails_box_add = emails_box_c.clone();
                        let checkbuttons_list_add = checkbuttons_list_c.clone();
                        let manual_entry_c = manual_entry.clone();
                        add_btn.connect_clicked(move |_| {
                            let text = manual_entry_c.text().to_string();
                            if !text.is_empty() && text.contains('@') {
                                let cb = CheckButton::with_label(&text);
                                cb.set_active(true);
                                emails_box_add.pack_start(&cb, false, false, 0);
                                checkbuttons_list_add.borrow_mut().push(cb);
                                emails_box_add.show_all();
                                manual_entry_c.set_text("");
                            }
                        });

                        let manual_entry_widget = manual_entry.clone();
                        manual_box.pack_start(&manual_entry_widget, true, true, 0);
                        manual_box.pack_start(&add_btn, false, false, 0);
                        emails_box_c.pack_start(&manual_box, false, false, 0);

                        emails_box_c.show_all();
                        btn_next_c.set_sensitive(true);
                        gtk::glib::ControlFlow::Break
                    } else {
                        gtk::glib::ControlFlow::Continue
                    }
                });

                let domain_c = domain_str;
                std::thread::spawn(move || {
                    let res = crawler::crawl_domain(domain_c);
                    let _ = tx.send(res);
                });
            }
        }
        3 => {
            let mut emails = Vec::new();
            for cb in checkbuttons_list_next.borrow().iter() {
                if cb.is_active() {
                    if let Some(lbl) = cb.label() {
                        emails.push(lbl.to_string());
                    }
                }
            }
            if emails.is_empty() {
                show_error_dialog(
                    wizard_next,
                    "Please select or add at least one email address.",
                );
                return;
            }
            *selected_emails_next.borrow_mut() = emails;

            notebook_next.set_current_page(Some(4));
            current_step_next.set(4);
            btn_next_next.set_label("Finish");

            let domain = selected_domain_next.borrow().clone();
            let user_email_val = entry_user_email_next.text().to_string();
            let user_username_val = entry_user_username_next.text().to_string();

            if let Some(conn) = shared_conn_next.borrow().as_ref() {
                let _ = crate::unsubscribe::db::save_credentials_for_domain(
                    conn,
                    &domain,
                    &user_username_val,
                    &user_email_val,
                );
                // Update the plain index so the browser can hint without decrypting the vault
                let mut idx = crate::util::credentials::CredentialIndex::load();
                idx.register(&domain);
            }

            let (subject, body) = email::build_gdpr_notice(
                user_name_next,
                user_id_next,
                &user_email_val,
                &user_username_val,
                &domain,
            );

            let buffer = text_preview_next.buffer().unwrap();
            buffer.set_text(&format!("Subject: {}\n\n{}", subject, body));
        }
        4 => {
            // Dispatch notice
            let domain = selected_domain_next.borrow().clone();
            let emails = selected_emails_next.borrow().clone();
            if !emails.is_empty() {
                let recipient = &emails[0];
                let user_email_val = entry_user_email_next.text().to_string();
                let user_username_val = entry_user_username_next.text().to_string();
                let (subject, body) = email::build_gdpr_notice(
                    user_name_next,
                    user_id_next,
                    &user_email_val,
                    &user_username_val,
                    &domain,
                );

                if let Some(smtp) = active_smtp_next.borrow().as_ref() {
                    match email::send_gdpr_smtp(smtp, recipient, &subject, &body) {
                        Ok(_) => {
                            show_info_dialog(
                                window_next,
                                "Success",
                                "GDPR Article 17 Erasure notice dispatched successfully via SMTP outbox.",
                            );
                        }
                        Err(e) => {
                            show_error_dialog(
                                window_next,
                                &format!(
                                    "Failed to send SMTP email: {}. Falling back to opening system mail client...",
                                    e
                                ),
                            );
                            let mailto = email::build_mailto_link(recipient, &subject, &body);
                            gtk::show_uri_on_window(Some(window_next), &mailto, 0).ok();
                        }
                    }
                } else {
                    let mailto = email::build_mailto_link(recipient, &subject, &body);
                    gtk::show_uri_on_window(Some(window_next), &mailto, 0).ok();
                }
            }

            registry_next.borrow_mut().add_notified(domain, emails);
            unsafe {
                wizard_next.destroy();
            }
        }
        5 => {
            // Reincident Report Generation
            if let Some(domain) = combo_notified_next.active_text() {
                let domain_str = domain.as_str().to_string();
                let recipient = dpo_email_entry_next.text().to_string();
                handle_report_generation(
                    window_next,
                    wizard_next,
                    &domain_str,
                    &recipient,
                    registry_next,
                    user_name_next,
                    user_id_next,
                    shared_conn_next,
                );
            }
        }
        _ => {}
    }
}
