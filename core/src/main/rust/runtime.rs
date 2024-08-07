use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::runtime::{Builder, Runtime};
use tracing::{error, info};

pub static RUNTIME: Lazy<RuntimeHolder> = Lazy::new(RuntimeHolder::new);

pub struct RuntimeHolder {
    runtime: MaybeUninit<Runtime>,
    is_initialized: AtomicBool,
}

impl RuntimeHolder {
    #[inline]
    fn new() -> RuntimeHolder {
        info!("Initializing runtime");
        RuntimeHolder {
            runtime: MaybeUninit::new(
                Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Unable to create tokio runtime"),
            ),
            is_initialized: AtomicBool::new(true),
        }
    }

    #[inline]
    pub(super) fn shutdown(&self) {
        if self.is_initialized.load(Ordering::Relaxed) {
            info!("Waiting for everything to quiet down");
            thread::sleep(Duration::from_millis(500));
            info!("Shutting down runtime");
            unsafe { self.runtime.assume_init_read() }.shutdown_timeout(Duration::from_secs(5));
            self.is_initialized.store(false, Ordering::Relaxed);
            info!("Runtime has been shut down");
        } else {
            error!("Runtime is already shut down");
        }
    }
}

impl Deref for RuntimeHolder {
    type Target = Runtime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        if self.is_initialized.load(Ordering::Relaxed) {
            unsafe { self.runtime.assume_init_ref() }
        } else {
            panic!("Runtime has been uninitialized");
        }
    }
}
