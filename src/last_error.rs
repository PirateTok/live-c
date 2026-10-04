use std::cell::RefCell;
use std::ffi::{c_char, CString};

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::from(c""));
}

pub fn set(message: &str) {
    let text = match CString::new(message.replace('\0', "\u{fffd}")) {
        Ok(text) => text,
        Err(e) => panic!("NUL survived replacement: {e}"),
    };
    LAST_ERROR.with(|slot| *slot.borrow_mut() = text);
}

#[no_mangle]
pub extern "C" fn piratetok_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}
