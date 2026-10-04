use piratetok_live_rs::helpers::profile_cache::ProfileCache;
use tokio::runtime::Runtime;

use crate::last_error;

pub struct PirateTokRuntime {
    pub rt: Runtime,
    pub profiles: ProfileCache,
}

#[no_mangle]
pub extern "C" fn piratetok_init() -> *mut PirateTokRuntime {
    match Runtime::new() {
        Ok(rt) => Box::into_raw(Box::new(PirateTokRuntime {
            rt,
            profiles: ProfileCache::new(),
        })),
        Err(e) => {
            tracing::error!(error = %e, "failed to create tokio runtime");
            last_error::set(&format!("failed to create runtime: {e}"));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_shutdown(rt: *mut PirateTokRuntime) {
    if rt.is_null() {
        return;
    }
    let runtime: Box<PirateTokRuntime> = Box::from_raw(rt);
    drop(runtime);
}
