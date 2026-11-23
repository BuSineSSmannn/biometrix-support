//! Biometrix Support build of the remote desktop core.
//!
//! Enabled with the `biometrix` cargo feature. The core runs headless on school PCs:
//! it is started by the Biometrix service, connects only to the Biometrix rendezvous
//! server and never shows its own connection manager window — the "support is
//! connected" notice is shown by the Biometrix desktop app instead.

use hbb_common::config;

/// Application name: config directory, log directory and IPC pipe names derive from it.
pub const APP_NAME: &str = "BiometrixSupport";

/// Rendezvous server (hbbs/hbbr) operated by Biometrix.
pub const RENDEZVOUS_SERVER: &str = match option_env!("BIOMETRIX_RENDEZVOUS_SERVER") {
    Some(server) => server,
    None => "biometrix-support.jasondev.uz",
};

/// Public key of the Biometrix rendezvous server (public by design: clients verify the server with it).
pub const RS_PUB_KEY: &str = match option_env!("BIOMETRIX_RS_PUB_KEY") {
    Some(key) => key,
    None => "xY0mk0TdIauLDR3RU8EQOquBNr84wjDUVfcMQXHzqkA=",
};

/// Must run before anything reads the config: the config path depends on the app name.
pub fn init() {
    *config::APP_NAME.write().unwrap() = APP_NAME.to_owned();
    *config::PROD_RENDEZVOUS_SERVER.write().unwrap() = RENDEZVOUS_SERVER.to_owned();
}

#[cfg(not(any(target_os = "android", target_os = "ios", feature = "flutter")))]
pub mod headless_cm {
    use crate::ui_cm_interface::{start_ipc, Client, ConnectionManager, InvokeUiCM};
    use hbb_common::log;

    /// Connection manager without a window: keeps the IPC side the server needs
    /// (file transfer, clipboard, chat routing) and only logs UI events.
    #[derive(Clone, Default)]
    struct HeadlessHandler;

    impl InvokeUiCM for HeadlessHandler {
        fn add_connection(&self, client: &Client) {
            log::info!(
                "support connection {} added: peer {}, authorized {}, file transfer {}",
                client.id,
                client.peer_id,
                client.authorized,
                client.is_file_transfer
            );
        }

        fn remove_connection(&self, id: i32, close: bool) {
            log::info!("support connection {} removed, close {}", id, close);
        }

        fn new_message(&self, id: i32, _text: String) {
            log::info!("support connection {} sent a chat message", id);
        }

        fn change_theme(&self, _dark: String) {}

        fn change_language(&self) {}

        fn show_elevation(&self, show: bool) {
            log::info!("elevation request, show {}", show);
        }

        fn update_voice_call_state(&self, client: &Client) {
            log::info!("voice call state changed for connection {}", client.id);
        }

        fn file_transfer_log(&self, action: &str, _log: &str) {
            log::info!("file transfer: {}", action);
        }
    }

    /// Blocks: serves the `_cm` IPC channel until the process is stopped.
    pub fn start() {
        log::info!("starting headless connection manager");
        start_ipc(ConnectionManager {
            ui_handler: HeadlessHandler,
        });
    }
}
