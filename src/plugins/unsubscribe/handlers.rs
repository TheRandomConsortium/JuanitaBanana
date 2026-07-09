#![allow(clippy::too_many_arguments)]
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText, Dialog, Entry, Notebook,
    RadioButton, Spinner,
};
use rusqlite::Connection;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::report_action::handle_report_generation;
use super::show_info_dialog;
use super::smtp_dialog::show_smtp_config_dialog;
use crate::unsubscribe::db::{PopConfig, SecureDbManager, SmtpConfig};
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
    entry_user_email: &Entry,
    entry_user_username: &Entry,
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
    let entry_user_email_next = entry_user_email.clone();
    let entry_user_username_next = entry_user_username.clone();
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
    let shared_conn_next = shared_conn.clone();
    let _shared_manager_next = shared_manager.clone();

    btn_next.connect_clicked(move |_| {
        let step = current_step_next.get();
        super::step_logic::handle_next_click(
            step,
            &window_next,
            &wizard_next,
            &notebook_next,
            &btn_back_next,
            &btn_next_next,
            &rb_unsub_next,
            &rb_report_next,
            &rb_curr_next,
            &rb_manual_next,
            &rb_search_next,
            &entry_manual_next,
            &entry_search_next,
            &combo_ddg_next,
            &combo_notified_next,
            &dpo_email_entry_next,
            &entry_user_email_next,
            &entry_user_username_next,
            &spinner_next,
            &emails_box_next,
            &text_preview_next,
            &current_step_next,
            &selected_domain_next,
            &selected_emails_next,
            &active_smtp_next,
            &checkbuttons_list_next,
            &shared_conn_next,
            &user_name_next,
            &user_id_next,
            &current_domain_next,
            &registry_next,
        );
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
    let shared_conn_gen = shared_conn.clone();
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
                &shared_conn_gen,
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
