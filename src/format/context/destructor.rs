use crate::ffi::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Mode {
    Input,
    Output,
}

pub struct Destructor {
    ptr: *mut AVFormatContext,
    mode: Mode,
    // Dropped after Drop::drop closes the native context, including close-time callbacks.
    interrupt: Option<crate::util::interrupt::Interrupt>,
}

impl Destructor {
    pub unsafe fn new(ptr: *mut AVFormatContext, mode: Mode) -> Self {
        Destructor {
            ptr,
            mode,
            interrupt: None,
        }
    }

    pub(crate) fn set_interrupt(&mut self, interrupt: crate::util::interrupt::Interrupt) {
        self.interrupt = Some(interrupt);
    }
}

unsafe impl Send for Destructor {}
unsafe impl Sync for Destructor {}

impl Drop for Destructor {
    fn drop(&mut self) {
        unsafe {
            match self.mode {
                Mode::Input => avformat_close_input(&mut self.ptr),

                Mode::Output => {
                    avio_close((*self.ptr).pb);
                    avformat_free_context(self.ptr);
                }
            }
        }
    }
}
