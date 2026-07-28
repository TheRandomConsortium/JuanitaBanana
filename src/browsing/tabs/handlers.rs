use webkit2gtk::{
    UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
    UserScriptInjectionTime, WebContextExt, WebView, WebViewExt,
};

use crate::fingerprint::spoof;
use crate::util::config::AppConfig;

pub fn setup_user_content_manager(ucm: &UserContentManager, config: &AppConfig) {
    let script = UserScript::new(
        spoof::anti_fingerprint_script(),
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&script);

    ucm.register_script_message_handler("juanita");
    let ad_script = UserScript::new(
        &crate::ad_intoxication::ad_intoxication_script(config),
        UserContentInjectedFrames::AllFrames,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&ad_script);

    let toxic_script = UserScript::new(
        &crate::util::ban::toxic_warning_script(config),
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &["juanita://*"],
    );
    ucm.add_script(&toxic_script);

    if config.guilt_trip_enabled {
        let guilt_script = UserScript::new(
            &crate::browsing::guilt::guilt_trip_script(config),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::Start,
            &[],
            &["juanita://*"],
        );
        ucm.add_script(&guilt_script);
    }

    if config.ai_slop_detection_enabled {
        let ai_slop_script = UserScript::new(
            &crate::privacy::ai_slop::ai_slop_detector_script(config),
            UserContentInjectedFrames::TopFrame,
            UserScriptInjectionTime::End,
            &[],
            &["juanita://*"],
        );
        ucm.add_script(&ai_slop_script);
    }
}

pub fn render_tls_error(wv: &WebView, failing_uri: &str) {
    if let Some(ctx) = wv.context() {
        ctx.clear_cache();
    }
    let http_uri = failing_uri.replace("https://", "http://");
    let html = include_root_str!(@templates, "tls.html")
        .replace(
            "{shared_css}",
            crate::browsing::internal::SHARED_CSS.as_str(),
        )
        .replace(
            "{{CERTBOT_IMG}}",
            &crate::util::image::get_juanita_certbot_b64(),
        )
        .replace("{{HTTP_URI}}", &http_uri);
    wv.load_html(&html, Some(failing_uri));
}

pub fn render_proxy_error(wv: &WebView, failing_uri: &str, error_message: &str) {
    let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
    let broken_pipe_img = crate::util::image::get_juanita_broken_pipe_b64();
    let error_html = include_root_str!(@templates, "proxy.html")
        .replace("{shared_css}", shared_css)
        .replace("{{BROKEN_PIPE_IMG}}", &broken_pipe_img)
        .replace("{{ERROR_MESSAGE}}", error_message);
    wv.load_html(&error_html, Some(failing_uri));
}
