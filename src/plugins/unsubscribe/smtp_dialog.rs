use gtk::prelude::*;
use gtk::{Dialog, DialogFlags, Entry, Grid, Label, ResponseType};
use rusqlite::Connection;
use std::cell::RefCell;
use std::rc::Rc;

use crate::unsubscribe::db::{self, PopConfig, SmtpConfig};

pub fn show_smtp_config_dialog<P: IsA<gtk::Window>>(
    parent: &P,
    active_smtp: Rc<RefCell<Option<SmtpConfig>>>,
    active_pop: Rc<RefCell<Option<PopConfig>>>,
    shared_conn: Rc<RefCell<Option<Connection>>>,
) {
    let smtp_dialog = Dialog::with_buttons(
        Some("Configure SMTP & POP Credentials"),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        &[
            ("Save", ResponseType::Accept),
            ("Cancel", ResponseType::Cancel),
        ],
    );
    let smtp_content = smtp_dialog.content_area();
    smtp_content.set_margin_start(15);
    smtp_content.set_margin_end(15);
    smtp_content.set_margin_top(15);
    smtp_content.set_margin_bottom(15);
    smtp_content.set_spacing(10);

    let grid = Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);

    // --- SMTP Section ---
    let smtp_header = Label::new(Some("SMTP Configuration (Outgoing)"));
    grid.attach(&smtp_header, 0, 0, 2, 1);

    grid.attach(&Label::new(Some("SMTP Server:")), 0, 1, 1, 1);
    let s_server = Entry::new();
    grid.attach(&s_server, 1, 1, 1, 1);

    grid.attach(&Label::new(Some("SMTP Port:")), 0, 2, 1, 1);
    let s_port = Entry::new();
    s_port.set_text("587");
    grid.attach(&s_port, 1, 2, 1, 1);

    grid.attach(&Label::new(Some("SMTP Username:")), 0, 3, 1, 1);
    let s_user = Entry::new();
    grid.attach(&s_user, 1, 3, 1, 1);

    grid.attach(&Label::new(Some("SMTP Password:")), 0, 4, 1, 1);
    let s_pass = Entry::new();
    s_pass.set_visibility(false);
    grid.attach(&s_pass, 1, 4, 1, 1);

    // --- POP Section ---
    let pop_header = Label::new(Some("POP3 Configuration (Incoming)"));
    grid.attach(&pop_header, 0, 5, 2, 1);

    grid.attach(&Label::new(Some("POP3 Server:")), 0, 6, 1, 1);
    let p_server = Entry::new();
    grid.attach(&p_server, 1, 6, 1, 1);

    grid.attach(&Label::new(Some("POP3 Port:")), 0, 7, 1, 1);
    let p_port = Entry::new();
    p_port.set_text("995");
    grid.attach(&p_port, 1, 7, 1, 1);

    grid.attach(&Label::new(Some("POP3 Username:")), 0, 8, 1, 1);
    let p_user = Entry::new();
    grid.attach(&p_user, 1, 8, 1, 1);

    grid.attach(&Label::new(Some("POP3 Password:")), 0, 9, 1, 1);
    let p_pass = Entry::new();
    p_pass.set_visibility(false);
    grid.attach(&p_pass, 1, 9, 1, 1);

    if let Some(cfg) = active_smtp.borrow().as_ref() {
        s_server.set_text(&cfg.server);
        s_port.set_text(&cfg.port.to_string());
        s_user.set_text(&cfg.user);
        s_pass.set_text(&cfg.pass);
    }
    if let Some(cfg) = active_pop.borrow().as_ref() {
        p_server.set_text(&cfg.server);
        p_port.set_text(&cfg.port.to_string());
        p_user.set_text(&cfg.user);
        p_pass.set_text(&cfg.pass);
    }

    smtp_content.pack_start(&grid, true, true, 0);
    smtp_dialog.show_all();

    if smtp_dialog.run() == ResponseType::Accept {
        let server = s_server.text().to_string();
        let port_str = s_port.text().to_string();
        let user = s_user.text().to_string();
        let pass = s_pass.text().to_string();

        if let Ok(port) = port_str.parse::<u16>() {
            if !server.is_empty() && !user.is_empty() && !pass.is_empty() {
                let new_cfg = SmtpConfig {
                    server,
                    port,
                    user,
                    pass,
                };
                *active_smtp.borrow_mut() = Some(new_cfg.clone());
                if let Some(conn_ref) = shared_conn.borrow().as_ref() {
                    let _ = db::save_smtp_config(conn_ref, &new_cfg);
                }
            }
        }

        let pop_server_str = p_server.text().to_string();
        let pop_port_str = p_port.text().to_string();
        let pop_user_str = p_user.text().to_string();
        let pop_pass_str = p_pass.text().to_string();

        if let Ok(port) = pop_port_str.parse::<u16>() {
            if !pop_server_str.is_empty() && !pop_user_str.is_empty() && !pop_pass_str.is_empty() {
                let new_cfg = PopConfig {
                    server: pop_server_str,
                    port,
                    user: pop_user_str,
                    pass: pop_pass_str,
                };
                *active_pop.borrow_mut() = Some(new_cfg.clone());
                if let Some(conn_ref) = shared_conn.borrow().as_ref() {
                    let _ = db::save_pop_config(conn_ref, &new_cfg);
                }
            }
        }
    }
    unsafe {
        smtp_dialog.destroy();
    }
}
