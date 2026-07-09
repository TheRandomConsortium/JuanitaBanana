#![allow(clippy::too_many_arguments)]
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, ComboBoxText, Dialog, DialogFlags, Entry, Label,
    Notebook, Orientation, RadioButton, Spinner,
};
use rusqlite::Connection;
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{WebView, WebViewExt};

use super::handlers;
use crate::unsubscribe::db::{PopConfig, SecureDbManager, SmtpConfig};
use crate::unsubscribe::registry::UnsubscribeRegistry;

pub fn show_unsubscribe_wizard(
    window: &ApplicationWindow,
    webview: &WebView,
    shared_conn: Rc<RefCell<Option<Connection>>>,
    shared_manager: Rc<RefCell<Option<SecureDbManager>>>,
    user_name: String,
    user_id: String,
    smtp_config: Option<SmtpConfig>,
    pop_config: Option<PopConfig>,
) {
    let registry = UnsubscribeRegistry::load();

    let wizard = Dialog::with_buttons(
        Some("Aggressive Unsubscribe Wizard 🍌"),
        Some(window),
        DialogFlags::DESTROY_WITH_PARENT,
        &[],
    );
    wizard.set_default_size(600, 450);

    let content_area = wizard.content_area();
    content_area.set_margin_start(15);
    content_area.set_margin_end(15);
    content_area.set_margin_top(15);
    content_area.set_margin_bottom(15);

    let notebook = Notebook::new();
    notebook.set_show_tabs(false);
    notebook.set_show_border(false);
    content_area.pack_start(&notebook, true, true, 0);

    // Page 0: Action Selection
    let page0 = GtkBox::new(Orientation::Vertical, 15);
    let title0 = Label::new(Some("Select Action"));
    title0.style_context().add_class("h2");
    page0.pack_start(&title0, false, false, 0);

    let rb_unsub = RadioButton::with_label("Unsubscribe New Domain");
    let rb_report = RadioButton::with_label_from_widget(&rb_unsub, "Report Reincident Domain");
    page0.pack_start(&rb_unsub, false, false, 0);
    page0.pack_start(&rb_report, false, false, 0);
    notebook.append_page(&page0, None::<&Label>);

    // Page 1: Domain Source Selection
    let page1 = GtkBox::new(Orientation::Vertical, 15);
    let title1 = Label::new(Some("Select Target Domain"));
    title1.style_context().add_class("h2");
    page1.pack_start(&title1, false, false, 0);

    let current_uri = webview.uri().unwrap_or_default().to_string();
    let current_domain = crate::browsing::browser::extract_domain(&current_uri);

    let rb_curr = RadioButton::with_label(&format!("Current Domain ({})", current_domain));
    let rb_manual = RadioButton::with_label_from_widget(&rb_curr, "Manual Domain Entry");
    let entry_manual = Entry::new();
    entry_manual.set_placeholder_text(Some("example.com"));

    let rb_search = RadioButton::with_label_from_widget(&rb_curr, "Search by Brand Name / Email");
    let entry_search = Entry::new();
    entry_search.set_placeholder_text(Some("Spammy Brand"));

    page1.pack_start(&rb_curr, false, false, 0);
    page1.pack_start(&rb_manual, false, false, 0);
    page1.pack_start(&entry_manual, false, false, 0);
    page1.pack_start(&rb_search, false, false, 0);
    page1.pack_start(&entry_search, false, false, 0);
    notebook.append_page(&page1, None::<&Label>);

    // Page 2: DDG Domain Selection
    let page2 = GtkBox::new(Orientation::Vertical, 15);
    let title2 = Label::new(Some("DuckDuckGo Search Results"));
    page2.pack_start(&title2, false, false, 0);
    let combo_ddg = ComboBoxText::new();
    page2.pack_start(&combo_ddg, false, false, 0);
    notebook.append_page(&page2, None::<&Label>);

    // Page 3: Crawler Progress & Email Verification
    let page3 = GtkBox::new(Orientation::Vertical, 15);
    let title3 = Label::new(Some("Verifying Contact Emails"));
    page3.pack_start(&title3, false, false, 0);
    let spinner = Spinner::new();
    let status_label = Label::new(Some("Scanning domain for privacy/DPO contact emails..."));
    page3.pack_start(&spinner, false, false, 0);
    page3.pack_start(&status_label, false, false, 0);

    let emails_box = GtkBox::new(Orientation::Vertical, 5);
    page3.pack_start(&emails_box, true, true, 0);

    let user_account_box = GtkBox::new(Orientation::Vertical, 5);
    let lbl_email = Label::new(Some("Your registered email on this domain (optional):"));
    lbl_email.set_halign(gtk::Align::Start);
    let entry_user_email = Entry::new();
    let lbl_username = Label::new(Some(
        "Your registered username/handle on this domain (optional):",
    ));
    lbl_username.set_halign(gtk::Align::Start);
    let entry_user_username = Entry::new();

    user_account_box.pack_start(&lbl_email, false, false, 0);
    user_account_box.pack_start(&entry_user_email, false, false, 0);
    user_account_box.pack_start(&lbl_username, false, false, 0);
    user_account_box.pack_start(&entry_user_username, false, false, 0);
    page3.pack_start(&user_account_box, false, false, 10);

    notebook.append_page(&page3, None::<&Label>);

    // Page 4: Send Notice Preview / Dispatch
    let page4 = GtkBox::new(Orientation::Vertical, 10);
    let title4 = Label::new(Some("Formal GDPR Article 17 Notice"));
    page4.pack_start(&title4, false, false, 0);

    let text_preview = gtk::TextView::new();
    text_preview.set_editable(false);
    text_preview.set_wrap_mode(gtk::WrapMode::Word);
    let scroll_preview = gtk::ScrolledWindow::builder()
        .child(&text_preview)
        .min_content_height(180)
        .build();
    page4.pack_start(&scroll_preview, true, true, 0);

    let dispatch_action_box = GtkBox::new(Orientation::Horizontal, 10);

    let copy_clipboard_btn = Button::with_label("Copy to Clipboard");
    dispatch_action_box.pack_start(&copy_clipboard_btn, true, true, 0);

    let config_smtp_btn = Button::with_label("Configure SMTP Outbox (Opt-In)");
    dispatch_action_box.pack_start(&config_smtp_btn, true, true, 0);

    page4.pack_start(&dispatch_action_box, false, false, 0);

    let dispatch_btn = Button::with_label("Send Notice");
    dispatch_btn.style_context().add_class("suggested-action");
    page4.pack_start(&dispatch_btn, false, false, 0);
    notebook.append_page(&page4, None::<&Label>);

    // Page 5: Report Reincident Domain
    let page5 = GtkBox::new(Orientation::Vertical, 15);
    let title5 = Label::new(Some("Report Reincident Brand"));
    title5.style_context().add_class("h2");
    page5.pack_start(&title5, false, false, 0);

    let combo_notified = ComboBoxText::new();
    let notified_list: Vec<String> = registry.notified_domains.keys().cloned().collect();
    for domain in &notified_list {
        combo_notified.append_text(domain);
    }
    if !notified_list.is_empty() {
        combo_notified.set_active(Some(0));
    }

    let dpo_email_entry = Entry::new();
    dpo_email_entry.set_placeholder_text(Some("dpo@company.com"));

    page5.pack_start(
        &Label::new(Some(
            "Select the brand that ignored your unsubscribe notice:",
        )),
        false,
        false,
        0,
    );
    page5.pack_start(&combo_notified, false, false, 0);
    page5.pack_start(
        &Label::new(Some("Recipient DPO/Legal Email address:")),
        false,
        false,
        0,
    );
    page5.pack_start(&dpo_email_entry, false, false, 0);

    let generate_btn = Button::with_label("Generate official DPA Complaint Report");
    generate_btn.style_context().add_class("destructive-action");
    page5.pack_start(&generate_btn, false, false, 0);
    notebook.append_page(&page5, None::<&Label>);

    // Wizard Navigation Buttons at bottom
    let button_box = GtkBox::new(Orientation::Horizontal, 10);
    button_box.set_halign(gtk::Align::End);
    let btn_back = Button::with_label("Back");
    let btn_next = Button::with_label("Next");
    let btn_cancel = Button::with_label("Cancel");

    button_box.pack_start(&btn_back, false, false, 0);
    button_box.pack_start(&btn_next, false, false, 0);
    button_box.pack_start(&btn_cancel, false, false, 0);
    content_area.pack_start(&button_box, false, false, 10);

    btn_back.set_sensitive(false);

    // State Variables
    let current_step = Rc::new(std::cell::Cell::new(0));
    let selected_domain = Rc::new(RefCell::new(String::new()));
    let selected_emails = Rc::new(RefCell::new(Vec::<String>::new()));
    let active_smtp = Rc::new(RefCell::new(smtp_config));
    let active_pop = Rc::new(RefCell::new(pop_config));
    let checkbuttons_list = Rc::new(RefCell::new(Vec::<gtk::CheckButton>::new()));

    // Wire up events
    handlers::setup_handlers(
        window,
        &wizard,
        &notebook,
        &btn_back,
        &btn_next,
        &btn_cancel,
        &rb_unsub,
        &rb_report,
        &rb_curr,
        &rb_manual,
        &rb_search,
        &entry_manual,
        &entry_search,
        &combo_ddg,
        &combo_notified,
        &dpo_email_entry,
        &entry_user_email,
        &entry_user_username,
        &spinner,
        &emails_box,
        &text_preview,
        &copy_clipboard_btn,
        &config_smtp_btn,
        &dispatch_btn,
        &generate_btn,
        current_step,
        selected_domain,
        selected_emails,
        active_smtp,
        active_pop,
        checkbuttons_list,
        shared_conn,
        shared_manager,
        user_name,
        user_id,
        current_domain,
        registry,
    );

    wizard.show_all();
}
