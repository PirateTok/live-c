use std::ffi::c_char;

use piratetok_live_rs::http::sigi::SigiProfile;
use serde_json::{json, Value};

use crate::codes::{fail_ffi, fail_live, Code};
use crate::ffi_str::{required_str, write_out, FfiError};
use crate::runtime::PirateTokRuntime;

#[no_mangle]
pub unsafe extern "C" fn piratetok_fetch_profile(
    rt: *mut PirateTokRuntime,
    username: *const c_char,
    out_json: *mut *mut c_char,
) -> i32 {
    let Some(runtime) = rt.as_ref() else {
        return fail_ffi(&FfiError::NullPointer);
    };
    let name = match required_str(username) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!(error = %e, "fetch_profile: bad username");
            return fail_ffi(&e);
        }
    };
    let profile = match runtime.rt.block_on(runtime.profiles.fetch(name)) {
        Ok(profile) => profile,
        Err(e) => {
            tracing::warn!(error = %e, "fetch_profile failed");
            return fail_live(&e);
        }
    };
    match write_out(out_json, &profile_json(&profile).to_string()) {
        Ok(()) => Code::OK,
        Err(e) => {
            tracing::warn!(error = %e, "fetch_profile: output");
            fail_ffi(&e)
        }
    }
}

pub fn profile_json(p: &SigiProfile) -> Value {
    json!({
        "user_id": p.user_id,
        "unique_id": p.unique_id,
        "nickname": p.nickname,
        "bio": p.bio,
        "avatar_thumb": p.avatar_thumb,
        "avatar_medium": p.avatar_medium,
        "avatar_large": p.avatar_large,
        "verified": p.verified,
        "private_account": p.private_account,
        "is_organization": p.is_organization,
        "room_id": p.room_id,
        "bio_link": p.bio_link,
        "follower_count": p.follower_count,
        "following_count": p.following_count,
        "heart_count": p.heart_count,
        "video_count": p.video_count,
        "friend_count": p.friend_count,
    })
}
