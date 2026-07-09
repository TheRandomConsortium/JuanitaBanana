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
use super::smtp_dialog::show_smtp_config_dialog;
use super::{show_error_dialog, show_info_dialog};
use crate::unsubscribe::crawler;
use crate::unsubscribe::db::{PopConfig, SecureDbManager, SmtpConfig};
use crate::unsubscribe::email;
use crate::unsubscribe::registry::UnsubscribeRegistry;

pub fn setup_handlers(
    window: &ApplicationWindow,
    wizard: &Dialog,
    notebook: &Notebook,
    btn_back: &Button,
    btn_next: &Button,
    btn_cancel: &Button,
    rb_unsub: &RadioButton,
    rb_report: &RadioButton,
    rb_curr: &RadioButton,
    rb_manual: &RadioButton,
    rb_search: &RadioButton,
    entry_manual: &Entry,
    entry_search: &Entry,
    combo_ddg: &ComboBoxText,
    combo_notified: &ComboBoxText,
    dpo_email_entry: &Entry,
    spinner: &Spinner,
    emails_box: &GtkBox,
    text_preview: &gtk::TextView,
    copy_clipboard_btn: &Button,
    config_smtp_btn: &Button,
    _dispatch_btn: &Button,
    generate_btn: &Button,
    current_step: Rc<Cell<u32>>,
    selected_domain: Rc<RefCell<String>>,
    selected_emails: Rc<RefCell<Vec<String>>>,
    active_smtp: Rc<RefCell<Option<SmtpConfig>>>,
    active_pop: Rc<RefCell<Option<PopConfig>>>,
    checkbuttons_list: Rc<RefCell<Vec<CheckButton>>>,
    shared_conn: Rc<RefCell<Option<Connection>>>,
    shared_manager: Rc<RefCell<Option<SecureDbManager>>>,
    user_name: String,
    user_id: String,
    current_domain: String,
    registry: UnsubscribeRegistry,
) {
    // Clipboard Copy Handler
    let text_preview_c = text_preview.clone();
    let window_c = window.clone();
    copy_clipboard_btn.connect_clicked(move |_| {
        if let Some(buf) = text_preview_c.buffer() {
            let (start, end) = buf.bounds();
            let text = buf
                .text(&start, &end, false)
                .unwrap_or_default()
                .to_string();
            let clipboard = gtk::Clipboard::get(&gtk::gdk::SELECTION_CLIPBOARD);
            clipboard.set_text(&text);
            show_info_dialog(&window_c, "Copied", "Notice copied to system clipboard!");
        }
    });

    // SMTP Config handler
    let window_smtp = window.clone();
    let active_smtp_conf = active_smtp.clone();
    let active_pop_conf = active_pop.clone();
    let shared_conn_smtp = shared_conn.clone();
    config_smtp_btn.connect_clicked(move |_| {
        show_smtp_config_dialog(
            &window_smtp,
            active_smtp_conf.clone(),
            active_pop_conf.clone(),
            shared_conn_smtp.clone(),
        );
    });

    // Next Button Clones
    let current_step_next = current_step.clone();
    let notebook_next = notebook.clone();
    let wizard_next = wizard.clone();
    let btn_back_next = btn_back.clone();
    let btn_next_next = btn_next.clone();
    let rb_unsub_next = rb_unsub.clone();
    let rb_report_next = rb_report.clone();
    let rb_curr_next = rb_curr.clone();
    let current_domain_next = current_domain;
    let rb_manual_next = rb_manual.clone();
    let entry_manual_next = entry_manual.clone();
    let rb_search_next = rb_search.clone();
    let entry_search_next = entry_search.clone();
    let combo_ddg_next = combo_ddg.clone();
    let combo_notified_next = combo_notified.clone();
    let dpo_email_entry_next = dpo_email_entry.clone();
    let selected_domain_next = selected_domain.clone();
    let spinner_next = spinner.clone();
    let emails_box_next = emails_box.clone();
    let checkbuttons_list_next = checkbuttons_list.clone();
    let registry_next = Rc::new(RefCell::new(registry.clone()));
    let selected_emails_next = selected_emails.clone();
    let user_name_next = user_name.clone();
    let user_id_next = user_id.clone();
    let text_preview_next = text_preview.clone();
    let active_smtp_next = active_smtp.clone();
    let window_next = window.clone();

    btn_next.connect_clicked(move |_| {
        let step = current_step_next.get();
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
                    current_domain_next.clone()
                } else if rb_manual_next.is_active() {
                    entry_manual_next.text().to_string()
                } else if rb_search_next.is_active() {
                    let query = entry_search_next.text().to_string();
                    if query.is_empty() { return; }
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

                if domain.is_empty() { return; }

                if registry_next.borrow().is_notified(&domain) {
                    show_error_dialog(&wizard_next, &format!("Warning: {} was already unsubscribed. Redirecting to Report Reincident page...", domain));
                    notebook_next.set_current_page(Some(5));
                    current_step_next.set(5);
                    btn_next_next.set_label("Finish");
                    let notified_list: Vec<String> = registry_next.borrow().notified_domains.keys().cloned().collect();
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

                *selected_domain_next.borrow_mut() = domain.clone();

                notebook_next.set_current_page(Some(3));
                current_step_next.set(3);
                btn_next_next.set_sensitive(false);

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
                            Label::new(Some("No contact emails found. Please enter one manually below:"))
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

                let domain_c = domain.clone();
                std::thread::spawn(move || {
                    let res = crawler::crawl_domain(domain_c);
                    let _ = tx.send(res);
                });
            }
            2 => {
                if let Some(domain) = combo_ddg_next.active_text() {
                    let domain_str = domain.as_str().to_string();

                    if registry_next.borrow().is_notified(&domain_str) {
                        show_error_dialog(&wizard_next, &format!("Warning: {} was already unsubscribed. Redirecting to Report Reincident page...", domain_str));
                        notebook_next.set_current_page(Some(5));
                        current_step_next.set(5);
                        btn_next_next.set_label("Finish");
                        let notified_list: Vec<String> = registry_next.borrow().notified_domains.keys().cloned().collect();
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

                    *selected_domain_next.borrow_mut() = domain_str.clone();

                    notebook_next.set_current_page(Some(3));
                    current_step_next.set(3);
                    btn_next_next.set_sensitive(false);

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
                                Label::new(Some("No contact emails found. Please enter one manually below:"))
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

                    let domain_c = domain_str.clone();
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
                    show_error_dialog(&wizard_next, "Please select or add at least one email address.");
                    return;
                }
                *selected_emails_next.borrow_mut() = emails;

                notebook_next.set_current_page(Some(4));
                current_step_next.set(4);
                btn_next_next.set_label("Finish");

                let domain = selected_domain_next.borrow().clone();
                let recipient = if !selected_emails_next.borrow().is_empty() {
                    selected_emails_next.borrow()[0].clone()
                } else {
                    String::new()
                };
                let (subject, body) = email::build_gdpr_notice(&user_name_next, &user_id_next, &recipient, &domain);

                let buffer = text_preview_next.buffer().unwrap();
                buffer.set_text(&format!("Subject: {}\n\n{}", subject, body));
            }
            4 => {
                // Dispatch notice
                let domain = selected_domain_next.borrow().clone();
                let emails = selected_emails_next.borrow().clone();
                if !emails.is_empty() {
                    let recipient = &emails[0];
                    let (subject, body) = email::build_gdpr_notice(&user_name_next, &user_id_next, recipient, &domain);

                    if let Some(smtp) = active_smtp_next.borrow().as_ref() {
                        match email::send_gdpr_smtp(smtp, recipient, &subject, &body) {
                            Ok(_) => {
                                show_info_dialog(&window_next, "Success", "GDPR Article 17 Erasure notice dispatched successfully via SMTP outbox.");
                            }
                            Err(e) => {
                                show_error_dialog(&window_next, &format!("Failed to send SMTP email: {}. Falling back to opening system mail client...", e));
                                let mailto = email::build_mailto_link(recipient, &subject, &body);
                                gtk::show_uri_on_window(Some(&window_next), &mailto, 0).ok();
                            }
                        }
                    } else {
                        let mailto = email::build_mailto_link(recipient, &subject, &body);
                        gtk::show_uri_on_window(Some(&window_next), &mailto, 0).ok();
                    }
                }

                registry_next.borrow_mut().add_notified(domain, emails);
                unsafe { wizard_next.destroy(); }
            }
            5 => {
                // Reincident Report Generation
                if let Some(domain) = combo_notified_next.active_text() {
                    let domain_str = domain.as_str().to_string();
                    let recipient = dpo_email_entry_next.text().to_string();
                    handle_report_generation(
                        &window_next,
                        &wizard_next,
                        &domain_str,
                        &recipient,
                        &registry_next,
                        &user_name_next,
                        &user_id_next
                    );
                }
            }
            _ => {}
        }
    });

    // Back Button Clicked
    let current_step_back = current_step.clone();
    let notebook_back = notebook.clone();
    let btn_back_back = btn_back.clone();
    let btn_next_back = btn_next.clone();

    btn_back.connect_clicked(move |_| {
        let step = current_step_back.get();
        match step {
            1 | 5 => {
                notebook_back.set_current_page(Some(0));
                current_step_back.set(0);
                btn_back_back.set_sensitive(false);
                btn_next_back.set_label("Next");
            }
            2 => {
                notebook_back.set_current_page(Some(1));
                current_step_back.set(1);
                btn_next_back.set_label("Next");
            }
            3 => {
                notebook_back.set_current_page(Some(1));
                current_step_back.set(1);
                btn_next_back.set_sensitive(true);
                btn_next_back.set_label("Next");
            }
            4 => {
                notebook_back.set_current_page(Some(3));
                current_step_back.set(3);
                btn_next_back.set_label("Next");
            }
            _ => {}
        }
    });

    // Cancel Button Clicked
    let wizard_cancel = wizard.clone();
    btn_cancel.connect_clicked(move |_| unsafe {
        wizard_cancel.destroy();
    });

    // Generate button on reincident page
    let combo_notified_gen = combo_notified.clone();
    let dpo_email_entry_gen = dpo_email_entry.clone();
    let registry_gen = Rc::new(RefCell::new(registry));
    let user_name_gen = user_name;
    let user_id_gen = user_id;
    let window_gen = window.clone();
    let wizard_gen = wizard.clone();
    generate_btn.connect_clicked(move |_| {
        if let Some(domain) = combo_notified_gen.active_text() {
            let domain_str = domain.as_str().to_string();
            let recipient = dpo_email_entry_gen.text().to_string();
            handle_report_generation(
                &window_gen,
                &wizard_gen,
                &domain_str,
                &recipient,
                &registry_gen,
                &user_name_gen,
                &user_id_gen,
            );
        }
    });

    // Wizard Destroy Handler -> Cleanup DB
    let shared_conn_close = shared_conn.clone();
    let shared_manager_close = shared_manager.clone();
    wizard.connect_destroy(move |_| {
        if let Some(conn) = shared_conn_close.borrow_mut().take() {
            if let Some(mut manager) = shared_manager_close.borrow_mut().take() {
                let _ = manager.save_and_close(conn);
            }
        }
    });
}
