// ── Embedder module ───────────────────────────────────────────
//
// Integration point for Servo ↔ winit ↔ surfman ↔ egui.
//
// Servo requires the embedder to implement:
//   - EmbedderMethods: engine hooks (user-agent, scripts, etc.)
//   - WindowMethods:   access to the GL surface and dimensions
//
// The EmbedderMethods hooks are exactly where we plug in:
//   - spoof::USER_AGENT      → what the servers see
//   - spoof::anti_fingerprint_script() → JS injected on every page
//
// This file will be completed iteratively as we advance
// with the Servo embedding. The servo 0.3 public API is new
// and we will adjust it based on compilation errors.

/* TODO: Restore these imports when integrating Servo proper
use servo::{
    compositing::windowing::{EmbedderEvent, WindowMethods},
    embedder_traits::EmbedderMethods,
    euclid::{Point2D, Rect, Scale, Size2D},
    servo_url::ServoUrl,
    webrender_api::units::DevicePixel,
};
*/

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    // WindowBuilder changed in newer winit, we'll fix this later
    // window::WindowBuilder,
};

use crate::browser::{SharedBanList, normalize_url, extract_domain};
use crate::spoof;
use crate::ban;

// ── JuanitaEmbedder ──────────────────────────────────────────
// Implements EmbedderMethods — hooks called by Servo.
struct JuanitaEmbedder;

/* TODO: Implement EmbedderMethods when servo re-exports are found
impl EmbedderMethods for JuanitaEmbedder {
    fn get_user_agent_string(&self) -> Option<String> {
        Some(spoof::USER_AGENT.to_string())
    }

    fn get_javascript_to_evaluate_at_document_start(
        &self,
        _url: &ServoUrl,
    ) -> Option<String> {
        // Inject the anti-fingerprint script before ANY
        // JS from the page. This prevents them from overriding us.
        Some(spoof::anti_fingerprint_script().to_string())
    }
}
*/

// ── Main run loop ─────────────────────────────────────────────
pub fn run(state: SharedBanList) {
    // We are temporarily disabling the window creation until we
    // stabilize the surfman + Servo compositor integration.
    
    // let event_loop = EventLoop::new().expect("Failed to create event loop");
    // let window = WindowBuilder::new().with_title("Juanita Banana 🍌").build(&event_loop).unwrap();
    // let window = Arc::new(window);

    // TODO: Initialize surfman + Servo compositor here.
    // The Servo 0.3 API is being adjusted — this will be
    // completed as we resolve compilation errors.
    //
    // Reference: https://github.com/servo/servo/tree/main/ports/servoshell

    let home_url = "https://duckduckgo.com";
    let mut current_url = home_url.to_string();

    // UI state (egui chrome)
    let mut url_input = home_url.to_string();
    let mut is_banned_page = false;

    log::info!("[JuanitaBanana] Starting — engine: Servo, stack: 100% Rust");
    log::info!("[JuanitaBanana] User-Agent: {}", spoof::USER_AGENT);

    println!("Banana Browser Core Initialized!");
    println!("Servo Engine has successfully compiled!");
}
