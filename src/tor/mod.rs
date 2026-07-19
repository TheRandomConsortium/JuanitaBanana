//! # `tor` module — INTERIM SUBPROCESS ARCHITECTURE
//!
//! > **⚠️ DEPRECATED — All public items in this module are marked `#[deprecated]`.**
//!
//! This module manages the `arti` subprocess daemon and the local SOCKS5 proxy helper.
//! **None of it will survive Phase 4.** The planned replacement is:
//!
//! | Current (deprecated) | Phase 4 replacement |
//! |---|---|
//! | `daemon.rs` — `arti` subprocess lifecycle | `arti-client` crate, in-process circuit management |
//! | `proxy.rs` — local SOCKS5 helper (port 9151) | `arti-client` native stream, no proxy port needed |
//! | `webcontext.rs` — proxy config on `WebContext` | May survive in reduced form to configure TLS/network settings; proxy logic removed |
//!
//! See [`docs/OVERLAY_NETWORKS.md`](../../docs/OVERLAY_NETWORKS.md) — Phase 4 section.

#[allow(deprecated)]
pub mod daemon;
#[allow(deprecated)]
pub mod proxy;
#[allow(deprecated)]
pub mod webcontext;

#[deprecated(
    since = "1.6.8",
    note = "Phase 4: replaced by arti-client in-process circuit management. \
            The arti subprocess daemon will be removed entirely."
)]
pub use daemon::{init_tor, is_tor_running, shutdown_tor, ARTI_SOCKS_PORT};

#[deprecated(
    since = "1.6.8",
    note = "Phase 4: proxy port configuration on WebContext will be removed once \
            arti-client intercepts connections natively. May partially survive for \
            other WebContext network settings."
)]
pub use webcontext::apply_tor_proxy;

#[deprecated(
    since = "1.6.8",
    note = "Phase 4: the local SOCKS5 helper (port 9151) is an interim workaround for \
            WebKit SOCKS5 IP-as-hostname failures with arti. Will be removed entirely \
            when arti-client intercepts WebKit connections in-process."
)]
pub use proxy::{start_local_proxy, LOCAL_PROXY_PORT};
