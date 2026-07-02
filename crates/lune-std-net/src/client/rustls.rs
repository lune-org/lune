use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};

use rustls::{ClientConfig, crypto::ring};

static PROVIDER_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn initialize_provider() {
    if !PROVIDER_INITIALIZED.load(Ordering::Relaxed) {
        PROVIDER_INITIALIZED.store(true, Ordering::Relaxed);
        // Only errors if already installed, which is fine
        ring::default_provider().install_default().ok();
    }
}

pub static CLIENT_CONFIG: LazyLock<Arc<ClientConfig>> = LazyLock::new(|| {
    initialize_provider();
    rustls::ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth()
        .into()
});
