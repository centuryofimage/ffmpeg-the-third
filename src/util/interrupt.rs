use std::panic;
use std::process;
use std::sync::Mutex;

use crate::ffi::*;
use libc::{c_int, c_void};

pub struct Interrupt {
    pub interrupt: AVIOInterruptCB,
    // Keep the callback at a stable address even when its owner moves.
    _closure: Box<Mutex<Box<dyn FnMut() -> bool + Send>>>,
}

extern "C" fn callback(opaque: *mut c_void) -> c_int {
    // The owning format destructor retains this allocation until native close finishes.
    let closure = unsafe { &*(opaque as *const Mutex<Box<dyn FnMut() -> bool + Send>>) };
    match panic::catch_unwind(panic::AssertUnwindSafe(|| (closure.lock().unwrap())())) {
        Ok(ret) => ret as c_int,
        Err(_) => process::abort(),
    }
}

pub fn new<F>(opaque: Box<F>) -> Interrupt
where
    F: FnMut() -> bool + Send + 'static,
{
    let mut closure: Box<Mutex<Box<dyn FnMut() -> bool + Send>>> = Box::new(Mutex::new(opaque));
    let interrupt_cb = AVIOInterruptCB {
        callback: Some(callback),
        opaque: (&mut *closure as *mut Mutex<Box<dyn FnMut() -> bool + Send>>).cast(),
    };
    Interrupt {
        interrupt: interrupt_cb,
        _closure: closure,
    }
}
