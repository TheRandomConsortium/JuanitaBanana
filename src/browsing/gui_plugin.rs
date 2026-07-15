use crate::ad_intoxication::AdIntoxicationEngine;
use gtk::prelude::*;
use gtk::{ApplicationWindow, ComboBoxText, Dialog, Entry, Label, Scale};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{HitTestResultExt, WebView, WebViewExt};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RightClickInfo {
    pub frame_url: String,
    pub target_src: String,
    pub target_href: String,
}

thread_local! {
    pub static LAST_RIGHT_CLICK: RefCell<Option<RightClickInfo>> = const {RefCell::new(None)};
    pub static ACTIVE_TAB: RefCell<Option<(WebView, Rc<AdIntoxicationEngine>)>> = const {RefCell::new(None)};
}

pub trait GuiPlugin {
    fn setup(
        &self,
        window: &ApplicationWindow,
        webview: &WebView,
        ad_engine: &AdIntoxicationEngine,
    );
}

pub struct AdIntoxicationPlugin;

impl GuiPlugin for AdIntoxicationPlugin {
    fn setup(
        &self,
        window: &ApplicationWindow,
        webview: &WebView,
        _ad_engine: &AdIntoxicationEngine,
    ) {
        let mark_as_ad_action = if let Some(act) = window.lookup_action("mark-as-ad") {
            act.downcast::<gtk::gio::SimpleAction>().unwrap()
        } else {
            let act = gtk::gio::SimpleAction::new(
                "mark-as-ad",
                Some(gtk::glib::VariantTy::new("s").unwrap()),
            );
            let window_act = window.clone();
            act.connect_activate(move |_, parameter| {
                if let Some(param) = parameter {
                    if let Some(json_str) = param.str() {
                        if let Ok(candidates) = serde_json::from_str::<Vec<String>>(json_str) {
                            if candidates.is_empty() { return; }

                            let (active_wv, active_ad_engine) = ACTIVE_TAB.with(|at| {
                                at.borrow().clone().expect("No active tab set!")
                            });

                            let dialog = Dialog::with_buttons(
                                Some("Confirm Ad Target"),
                                Some(&window_act),
                                gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
                                &[("Accept", gtk::ResponseType::Accept), ("Cancel", gtk::ResponseType::Cancel)]
                            );
                            let content_area = dialog.content_area();
                            content_area.set_margin_start(15);
                            content_area.set_margin_end(15);
                            content_area.set_margin_top(15);
                            content_area.set_margin_bottom(15);
                            content_area.set_spacing(10);

                            let combo = ComboBoxText::new();
                            if candidates.len() == 1 {
                                let msg = format!("Are you sure you want to block & poison target: {}?", candidates[0]);
                                let label = Label::new(Some(&msg));
                                content_area.pack_start(&label, false, false, 0);
                                combo.append_text(&candidates[0]);
                                combo.set_active(Some(0));
                            } else {
                                let label = Label::new(Some("Select the target URL/domain to block & poison:"));
                                content_area.pack_start(&label, false, false, 0);
                                for cand in &candidates {
                                    combo.append_text(cand);
                                }
                                combo.set_active(Some(0));
                                content_area.pack_start(&combo, false, false, 0);
                            }

                            let phase = std::rc::Rc::new(std::cell::Cell::new(1));
                            let selected_uri = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
                            let chosen_depth = std::rc::Rc::new(std::cell::Cell::new(5));

                            let phase_resp = phase.clone();
                            let selected_uri_resp = selected_uri.clone();
                            let chosen_depth_resp = chosen_depth.clone();
                            let combo_clone = combo.clone();
                            let ad_engine_response = active_ad_engine.clone();
                            let main_wv_response = active_wv.clone();

                        dialog.connect_response(move |dialog_widget, response| {
                            let current_phase = phase_resp.get();
                            if response == gtk::ResponseType::Cancel || response == gtk::ResponseType::DeleteEvent {
                                // Cancel/Close: restore temporary hides and destroy
                                let restore_js = r##"
                                    (function() {
                                        document.querySelectorAll('[data-juanita-temp-hide]').forEach(el => {
                                            el.style.display = el.dataset.juanitaOrgDisplay || '';
                                            delete el.dataset.juanitaTempHide;
                                            delete el.dataset.juanitaOrgDisplay;
                                        });
                                    })();
                                "##;
                                #[allow(deprecated)]
                                main_wv_response.run_javascript(restore_js, None::<&webkit2gtk::gio::Cancellable>, |_| {});
                                unsafe { dialog_widget.destroy(); }
                                return;
                            }

                            if response == gtk::ResponseType::Accept {
                                if current_phase == 1 {
                                    if let Some(uri) = combo_clone.active_text() {
                                        let uri_str = uri.as_str().to_string();
                                        *selected_uri_resp.borrow_mut() = uri_str.clone();

                                        // Learn the domain
                                        let domain = crate::browsing::browser::extract_domain(&uri_str);
                                        ad_engine_response.learn_ad_domain(domain.clone());

                                        // Transition to Phase 2
                                        phase_resp.set(2);

                                        // Clear content area
                                        let content_area = dialog_widget.content_area();
                                        content_area.foreach(|w| content_area.remove(w));

                                        // Build Phase 2 UI
                                        let label = Label::new(Some("Step 2: Adjust depth slider until the ad container disappears."));
                                        content_area.pack_start(&label, false, false, 0);

                                        let config_temp = crate::util::config::AppConfig::load();
                                        let scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 10.0, 1.0);
                                        let default_depth = config_temp.ad_intox_max_depth;
                                        chosen_depth_resp.set(default_depth);
                                        scale.set_value(default_depth as f64);

                                        let depth_label = Label::new(Some(&format!("Current Depth: {}", default_depth)));
                                        content_area.pack_start(&depth_label, false, false, 0);
                                        content_area.pack_start(&scale, false, false, 0);

                                        let wv_scale = main_wv_response.clone();
                                        let uri_scale = uri_str.clone();
                                        let depth_label_clone = depth_label.clone();
                                        let chosen_depth_scale = chosen_depth_resp.clone();

                                        scale.connect_value_changed(move |s| {
                                            let depth = s.value() as usize;
                                            chosen_depth_scale.set(depth);
                                            depth_label_clone.set_text(&format!("Current Depth: {}", depth));
                                            let js = format!(
                                                r##"
                                                (function() {{
                                                    const uri = "{}";
                                                    const depth = {};

                                                    document.querySelectorAll('[data-juanita-temp-hide]').forEach(el => {{
                                                        el.style.display = el.dataset.juanitaOrgDisplay || '';
                                                        delete el.dataset.juanitaTempHide;
                                                        delete el.dataset.juanitaOrgDisplay;
                                                    }});

                                                    if (depth === 0) return;

                                                    const elements = document.querySelectorAll('iframe, img, a, script');
                                                    elements.forEach(el => {{
                                                        const src = el.getAttribute('src') || '';
                                                        const href = el.getAttribute('href') || '';
                                                        const matches_src = src && (src.includes(uri) || uri.includes(src));
                                                        const matches_href = href && (href.includes(uri) || uri.includes(href));
                                                        if (matches_src || matches_href) {{
                                                            let parent = el.parentElement;
                                                            for (let i = 1; i < depth; i++) {{
                                                                if (parent && parent.parentElement) {{
                                                                    parent = parent.parentElement;
                                                                }}
                                                            }}
                                                            if (parent) {{
                                                                parent.dataset.juanitaTempHide = "true";
                                                                parent.dataset.juanitaOrgDisplay = parent.style.display;
                                                                parent.style.display = "none";
                                                            }}
                                                        }}
                                                    }});
                                                }})();
                                                "##,
                                                uri_scale, depth
                                            );
                                            #[allow(deprecated)]
                                            wv_scale.run_javascript(&js, None::<&webkit2gtk::gio::Cancellable>, |_| {});
                                        });

                                        // Show everything
                                        dialog_widget.show_all();

                                        // Trigger the initial hide js at default_depth
                                        let init_js = format!(
                                            r##"
                                            (function() {{
                                                const uri = "{}";
                                                const depth = {};
                                                const elements = document.querySelectorAll('iframe, img, a, script');
                                                elements.forEach(el => {{
                                                    const src = el.getAttribute('src') || '';
                                                    const href = el.getAttribute('href') || '';
                                                    const matches_src = src && (src.includes(uri) || uri.includes(src));
                                                    const matches_href = href && (href.includes(uri) || uri.includes(href));
                                                    if (matches_src || matches_href) {{
                                                        let parent = el.parentElement;
                                                        for (let i = 1; i < depth; i++) {{
                                                            if (parent && parent.parentElement) {{
                                                                parent = parent.parentElement;
                                                            }}
                                                        }}
                                                        if (parent) {{
                                                            parent.dataset.juanitaTempHide = "true";
                                                            parent.dataset.juanitaOrgDisplay = parent.style.display;
                                                            parent.style.display = "none";
                                                        }}
                                                    }}
                                                }});
                                            }})();
                                            "##,
                                            uri_str, default_depth
                                        );
                                        #[allow(deprecated)]
                                        main_wv_response.run_javascript(&init_js, None::<&webkit2gtk::gio::Cancellable>, |_| {});
                                    }
                                } else if current_phase == 2 {
                                    // Accept clicked in Phase 2
                                    let depth_val = chosen_depth_resp.get();
                                    let uri_val = selected_uri_resp.borrow().clone();

                                    // Query parent details (classes & ID) from the web page
                                    let js_query = format!(
                                        r##"
                                        (function() {{
                                            const uri = "{}";
                                            const depth = {};
                                            const elements = document.querySelectorAll('iframe, img, a, script');
                                            let targetParent = null;
                                            for (let el of elements) {{
                                                const src = el.getAttribute('src') || '';
                                                const href = el.getAttribute('href') || '';
                                                const matches_src = src && (src.includes(uri) || uri.includes(src));
                                                const matches_href = href && (href.includes(uri) || uri.includes(href));
                                                if (matches_src || matches_href) {{
                                                    let parent = el.parentElement;
                                                    for (let i = 1; i < depth; i++) {{
                                                        if (parent && parent.parentElement) {{
                                                            parent = parent.parentElement;
                                                        }}
                                                    }}
                                                    if (parent) {{
                                                        targetParent = parent;
                                                        break;
                                                    }}
                                                }}
                                            }}
                                            if (targetParent) {{
                                                return JSON.stringify({{
                                                    id: targetParent.id || "",
                                                    classes: Array.from(targetParent.classList)
                                                }});
                                            }}
                                            return "null";
                                        }})();
                                        "##,
                                        uri_val, depth_val
                                    );

                                    let dialog_widget_phase3 = dialog_widget.clone();
                                    let phase_phase3 = phase_resp.clone();

                                    // Set dialog loading state
                                    let content_area = dialog_widget.content_area();
                                    content_area.foreach(|w| content_area.remove(w));
                                    let loading_label = Label::new(Some("Querying DOM structure, please wait..."));
                                    content_area.pack_start(&loading_label, false, false, 0);
                                    dialog_widget.show_all();

                                    #[allow(deprecated)]
                                    main_wv_response.run_javascript(&js_query, None::<&webkit2gtk::gio::Cancellable>, move |res| {
                                        let mut detected_id = String::new();
                                        let mut detected_classes = Vec::new();
                                        if let Ok(js_res) = res {
                                            if let Some(val) = js_res.js_value() {
                                                let json_str = val.to_string();
                                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                                    let val_to_parse = if parsed.is_string() {
                                                        serde_json::from_str::<serde_json::Value>(parsed.as_str().unwrap()).unwrap_or(parsed)
                                                    } else {
                                                        parsed
                                                    };
                                                    if let Some(id_str) = val_to_parse["id"].as_str() {
                                                        detected_id = id_str.to_string();
                                                    }
                                                    if let Some(arr) = val_to_parse["classes"].as_array() {
                                                        detected_classes = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                                    }
                                                }
                                            }
                                        }

                                        // Clear content area again to build Phase 3
                                        content_area.foreach(|w| content_area.remove(w));

                                        phase_phase3.set(3);

                                        let label = Label::new(Some("Step 3: Edit the regular expression rule for future matches."));
                                        content_area.pack_start(&label, false, false, 0);

                                        let mut info_msg = format!("Detected parent container at depth {}:\n", depth_val);
                                        if !detected_id.is_empty() {
                                            info_msg.push_str(&format!("  - ID: #{}\n", detected_id));
                                        }
                                        if !detected_classes.is_empty() {
                                            info_msg.push_str(&format!("  - Classes: .{}\n", detected_classes.join(".")));
                                        }
                                        if detected_id.is_empty() && detected_classes.is_empty() {
                                            info_msg.push_str("  - No ID or classes detected (anonymous element).\n");
                                        }

                                        let info_label = Label::new(Some(&info_msg));
                                        content_area.pack_start(&info_label, false, false, 0);

                                        let config_temp = crate::util::config::AppConfig::load();
                                        let current_regex = config_temp.ad_intox_regex.clone();

                                        let mut suggested_regex = current_regex.clone();
                                        let mut suggestions = Vec::new();
                                        if !detected_id.is_empty() && !current_regex.contains(&detected_id) {
                                            suggestions.push(detected_id.clone());
                                        }
                                        for cls in &detected_classes {
                                            if !current_regex.contains(cls) && !cls.contains("sz-") && !cls.contains("col-") && cls.len() > 2 {
                                                suggestions.push(cls.clone());
                                            }
                                        }

                                        if !suggestions.is_empty() {
                                            let suggestion_str = suggestions.join("|");
                                            suggested_regex.push_str(&format!("|{}", suggestion_str));
                                            let suggestion_label = Label::new(Some(&format!("Suggested append: |{}", suggestion_str)));
                                            content_area.pack_start(&suggestion_label, false, false, 0);
                                        }

                                        let entry = Entry::new();
                                        entry.set_text(&suggested_regex);
                                        content_area.pack_start(&entry, false, false, 0);

                                        dialog_widget_phase3.show_all();
                                    });
                                } else if current_phase == 3 {
                                    // Accept clicked in Phase 3
                                    let content_area = dialog_widget.content_area();
                                    let mut new_regex = String::new();
                                    for w in content_area.children() {
                                        if let Ok(entry_widget) = w.downcast::<Entry>() {
                                            new_regex = entry_widget.text().to_string();
                                            break;
                                        }
                                    }

                                    let mut config_save = crate::util::config::AppConfig::load();
                                    if !new_regex.is_empty() {
                                        config_save.ad_intox_regex = new_regex;
                                    }
                                    config_save.ad_intox_max_depth = chosen_depth_resp.get();
                                    config_save.save();

                                    let remove_js = r##"
                                        (function() {
                                            document.querySelectorAll('[data-juanita-temp-hide]').forEach(el => {
                                                el.remove();
                                            });
                                        })();
                                    "##;
                                    #[allow(deprecated)]
                                    main_wv_response.run_javascript(remove_js, None::<&webkit2gtk::gio::Cancellable>, |_| {});

                                    unsafe { dialog_widget.destroy(); }
                                }
                            }
                        });
                        dialog.show_all();
                    }
                }
            }
            });
            window.add_action(&act);
            act
        };

        let action_for_menu = mark_as_ad_action.clone();
        webview.connect_context_menu(move |_wv, menu, _event, hit_test| {
            let mut candidates = Vec::new();
            if let Some(l) = hit_test.link_uri() {
                candidates.push(l.to_string());
            }
            if let Some(i) = hit_test.image_uri() {
                candidates.push(i.to_string());
            }
            if let Some(m) = hit_test.media_uri() {
                candidates.push(m.to_string());
            }

            LAST_RIGHT_CLICK.with(|rc| {
                let mut rc_borrow = rc.borrow_mut();
                if let Some(info) = rc_borrow.take() {
                    if !info.frame_url.is_empty() && info.frame_url != "about:blank" {
                        candidates.push(info.frame_url);
                    }
                    if !info.target_src.is_empty() {
                        candidates.push(info.target_src);
                    }
                    if !info.target_href.is_empty() {
                        candidates.push(info.target_href);
                    }
                }
            });

            candidates.sort();
            candidates.dedup();

            if !candidates.is_empty() {
                if let Ok(target_str) = serde_json::to_string(&candidates) {
                    use webkit2gtk::ContextMenuExt;
                    let target_variant = target_str.to_variant();
                    let item = webkit2gtk::ContextMenuItem::from_gaction(
                        &action_for_menu,
                        "Mark as Ad (Learn & Remove)",
                        Some(&target_variant),
                    );
                    menu.append(&item);
                }
            }
            false
        });
    }
}
