#[cfg(feature = "ffmpeg_8_1")]
pub use crate::util::format::AlphaMode;
pub use crate::util::format::{pixel, Pixel};
pub use crate::util::format::{sample, Sample};

use crate::util::interrupt;

pub mod stream;

pub mod chapter;

pub mod context;
pub use self::context::Context;

pub mod format;
pub use self::format::{flag, Flags};
pub use self::format::{Input, Output};

pub mod network;

use std::ffi::{CString, OsStr};
use std::ptr;

use crate::ffi::*;
use crate::utils;
use crate::{AsMutPtr, Error};

pub fn version() -> u32 {
    unsafe { avformat_version() }
}

pub fn configuration() -> &'static str {
    unsafe { utils::str_from_c_ptr(avformat_configuration()) }
}

pub fn license() -> &'static str {
    unsafe { utils::str_from_c_ptr(avformat_license()) }
}

pub fn input<P: AsRef<OsStr>>(path_or_url: P) -> Result<context::Input, Error> {
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);

        match avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut()) {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

pub fn input_with_dictionary<P, Dict>(
    path_or_url: P,
    mut options: Dict,
) -> Result<context::Input, Error>
where
    Dict: AsMutPtr<*mut AVDictionary>,
    P: AsRef<OsStr>,
{
    unsafe {
        let mut ps = ptr::null_mut();
        let path = from_os_str(path_or_url);
        let res = avformat_open_input(
            &mut ps,
            path.as_ptr(),
            ptr::null_mut(),
            options.as_mut_ptr(),
        );

        match res {
            0 => match avformat_find_stream_info(ps, ptr::null_mut()) {
                r if r >= 0 => Ok(context::Input::wrap(ps)),
                e => {
                    avformat_close_input(&mut ps);
                    Err(Error::from(e))
                }
            },

            e => Err(Error::from(e)),
        }
    }
}

/// Open and probe an input with an owned interruption callback.
///
/// The callback may run on a native worker thread. It must own its captures and
/// is released after the native input closes, including on open/probe failure.
/// As with other interrupt callbacks in this crate, a panic aborts the process.
pub fn input_with_interrupt<P, F>(path_or_url: P, closure: F) -> Result<context::Input, Error>
where
    P: AsRef<OsStr>,
    F: FnMut() -> bool + Send + 'static,
{
    input_with_dictionary_and_interrupt(
        path_or_url,
        crate::Dictionary::new(),
        &crate::Dictionary::new(),
        closure,
    )
}

/// Open an input with format options, per-stream probe options, and an owned
/// interruption callback. For example, format options may set `probesize`,
/// `analyzeduration`, or `protocol_whitelist`; probe options may set `threads`.
/// Probe options apply to streams present after opening (see `find_stream_info`).
/// Interior NUL bytes in the path return `EINVAL`.
///
/// The callback is serialized, may run on a native worker thread, and remains
/// alive through input destruction. A callback panic aborts the process.
pub fn input_with_dictionary_and_interrupt<P, F>(
    path_or_url: P,
    mut options: crate::Dictionary,
    probe_options: &crate::Dictionary,
    closure: F,
) -> Result<context::Input, Error>
where
    P: AsRef<OsStr>,
    F: FnMut() -> bool + Send + 'static,
{
    let path = CString::new(path_or_url.as_ref().as_encoded_bytes()).map_err(|_| Error::Other {
        errno: libc::EINVAL,
    })?;
    let interrupt = interrupt::new(Box::new(closure));
    // Keep the callback owner local until opening succeeds. FFmpeg can invoke it
    // while opening, closing after failure, or probing an already-owned input.
    unsafe {
        let mut raw = avformat_alloc_context();
        if raw.is_null() {
            return Err(Error::Other {
                errno: libc::ENOMEM,
            });
        }
        (*raw).interrupt_callback = interrupt.interrupt;
        let result =
            avformat_open_input(&mut raw, path.as_ptr(), ptr::null(), options.as_mut_ptr());
        if result < 0 {
            avformat_close_input(&mut raw);
            return Err(Error::from(result));
        }
        let mut input = context::Input::wrap(raw);
        input.retain_interrupt(interrupt);
        input.find_stream_info(probe_options)?;
        Ok(input)
    }
}

fn from_os_str(path_or_url: impl AsRef<OsStr>) -> CString {
    CString::new(path_or_url.as_ref().as_encoded_bytes()).unwrap()
}

fn alloc_context(
    format_name: *const libc::c_char,
    filename: *const libc::c_char,
) -> Result<context::Output, Error> {
    let mut ps = ptr::null_mut();

    unsafe {
        let res = avformat_alloc_output_context2(&mut ps, ptr::null(), format_name, filename);
        if res >= 0 {
            Ok(context::Output::wrap(ps))
        } else {
            Err(Error::from(res))
        }
    }
}

fn open_context_write(
    ctx: &mut context::Output,
    filename: *const libc::c_char,
    opts: *mut *mut AVDictionary,
) -> Result<(), Error> {
    let res = unsafe {
        avio_open2(
            &mut (*ctx.as_mut_ptr()).pb,
            filename,
            AVIO_FLAG_WRITE,
            ptr::null(),
            opts,
        )
    };

    if res >= 0 {
        Ok(())
    } else {
        Err(Error::from(res))
    }
}

pub fn output<P: AsRef<OsStr>>(path_or_url: P) -> Result<context::Output, Error> {
    let filename = from_os_str(path_or_url);
    let mut ctx = alloc_context(ptr::null(), filename.as_ptr())?;

    if !ctx.format().flags().contains(Flags::NO_FILE) {
        open_context_write(&mut ctx, filename.as_ptr(), ptr::null_mut())?;
    }

    Ok(ctx)
}

pub fn output_with<P, Dict>(path_or_url: P, mut options: Dict) -> Result<context::Output, Error>
where
    P: AsRef<OsStr>,
    Dict: AsMutPtr<*mut AVDictionary>,
{
    let path = from_os_str(path_or_url);
    let mut ctx = alloc_context(ptr::null(), path.as_ptr())?;

    if !ctx.format().flags().contains(Flags::NO_FILE) {
        open_context_write(&mut ctx, path.as_ptr(), options.as_mut_ptr())?;
    }

    Ok(ctx)
}

pub fn output_as<P: AsRef<OsStr>>(path_or_url: P, format: &str) -> Result<context::Output, Error> {
    let path = from_os_str(path_or_url);
    let format = CString::new(format).unwrap();
    let mut ctx = alloc_context(format.as_ptr(), path.as_ptr())?;

    if !ctx.format().flags().contains(Flags::NO_FILE) {
        open_context_write(&mut ctx, path.as_ptr(), ptr::null_mut())?;
    }

    Ok(ctx)
}

pub fn output_as_with<P, Dict>(
    path_or_url: P,
    format: &str,
    mut options: Dict,
) -> Result<context::Output, Error>
where
    P: AsRef<OsStr>,
    Dict: AsMutPtr<*mut AVDictionary>,
{
    let path = from_os_str(path_or_url);
    let format = CString::new(format).unwrap();
    let mut ctx = alloc_context(format.as_ptr(), path.as_ptr())?;

    if !ctx.format().flags().contains(Flags::NO_FILE) {
        open_context_write(&mut ctx, path.as_ptr(), options.as_mut_ptr())?;
    }

    Ok(ctx)
}
