#![cfg(target_os = "macos")]
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

const DEFAULT_FILE_DESCRIPTOR_LIMIT: u64 = 1_048_575;

#[napi(object)]
pub struct FileDescriptorLimitResult {
    pub before_soft: i64,
    pub before_hard: i64,
    pub after_soft: i64,
    pub after_hard: i64,
    pub requested_soft: i64,
}

#[napi]
pub fn raise_file_descriptor_limit(target_soft: Option<i64>) -> Result<FileDescriptorLimitResult> {
    let requested_soft = target_soft
        .and_then(|value| u64::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_FILE_DESCRIPTOR_LIMIT);

    let before = get_file_descriptor_limit()?;
    let next_soft = requested_soft.min(before.rlim_max);

    if before.rlim_cur < next_soft {
        let next = libc::rlimit {
            rlim_cur: next_soft,
            rlim_max: before.rlim_max,
        };
        let result = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &next) };
        if result != 0 {
            return Err(Error::from_reason(format!(
                "setrlimit(RLIMIT_NOFILE) failed: {}",
                std::io::Error::last_os_error()
            )));
        }
    }

    let after = get_file_descriptor_limit()?;
    Ok(FileDescriptorLimitResult {
        before_soft: rlim_to_i64(before.rlim_cur),
        before_hard: rlim_to_i64(before.rlim_max),
        after_soft: rlim_to_i64(after.rlim_cur),
        after_hard: rlim_to_i64(after.rlim_max),
        requested_soft: rlim_to_i64(requested_soft),
    })
}

fn get_file_descriptor_limit() -> Result<libc::rlimit> {
    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let result = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) };
    if result != 0 {
        return Err(Error::from_reason(format!(
            "getrlimit(RLIMIT_NOFILE) failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(limit)
}

fn rlim_to_i64(value: libc::rlim_t) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
