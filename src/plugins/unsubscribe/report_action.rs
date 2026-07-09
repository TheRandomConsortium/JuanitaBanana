use gtk::prelude::*;
use gtk::Dialog;
use std::cell::RefCell;
use std::rc::Rc;

use super::{show_error_dialog, show_info_dialog};
use crate::unsubscribe::registry::UnsubscribeRegistry;
use crate::unsubscribe::report;

pub fn handle_report_generation<P: IsA<gtk::Window>>(
    parent_window: &P,
    wizard: &Dialog,
    domain_str: &str,
    recipient: &str,
    registry: &Rc<RefCell<UnsubscribeRegistry>>,
    user_name: &str,
    user_id: &str,
) {
    if recipient.is_empty() || !recipient.contains('@') {
        show_error_dialog(wizard, "Please enter a valid DPO recipient email.");
        return;
    }

    let mut notified_date = String::new();
    let mut emails_used = Vec::new();
    if let Some(entry) = registry.borrow().get_domain(domain_str) {
        notified_date = entry.notified_date.clone();
        emails_used = entry.emails_used.clone();
    }

    match report::generate_reincidence_report(
        user_name,
        user_id,
        recipient,
        domain_str,
        &notified_date,
        &emails_used,
        recipient,
    ) {
        Ok(path) => {
            show_info_dialog(
                parent_window,
                "Complaint Report Generated",
                &format!("The formal GDPR Supervisory Authority complaint has been saved to your downloads at:\n\n{}", path.to_string_lossy())
            );
            registry.borrow_mut().mark_reincident(domain_str);
            unsafe {
                wizard.destroy();
            }
        }
        Err(e) => {
            show_error_dialog(
                parent_window,
                &format!("Failed to generate complaint report: {}", e),
            );
        }
    }
}
