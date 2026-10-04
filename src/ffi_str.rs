use std::ffi::{c_char, CStr, CString, NulError};
use std::fmt;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum FfiError {
    NullPointer,
    Utf8(Utf8Error),
    Nul(NulError),
    Json(serde_json::Error),
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiError::NullPointer => f.write_str("null pointer argument"),
            FfiError::Utf8(e) => write!(f, "argument is not valid utf-8: {e}"),
            FfiError::Nul(e) => write!(f, "string contains NUL: {e}"),
            FfiError::Json(e) => write!(f, "json: {e}"),
        }
    }
}

impl std::error::Error for FfiError {}

pub enum CStrArg<'a> {
    Given(&'a str),
    Absent,
}

pub unsafe fn required_str<'a>(p: *const c_char) -> Result<&'a str, FfiError> {
    if p.is_null() {
        return Err(FfiError::NullPointer);
    }
    CStr::from_ptr(p).to_str().map_err(FfiError::Utf8)
}

pub unsafe fn optional_str<'a>(p: *const c_char) -> Result<CStrArg<'a>, FfiError> {
    if p.is_null() {
        return Ok(CStrArg::Absent);
    }
    Ok(CStrArg::Given(required_str(p)?))
}

pub unsafe fn write_out(out: *mut *mut c_char, value: &str) -> Result<(), FfiError> {
    if out.is_null() {
        return Err(FfiError::NullPointer);
    }
    *out = CString::new(value).map_err(FfiError::Nul)?.into_raw();
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let owned: CString = CString::from_raw(s);
    drop(owned);
}
