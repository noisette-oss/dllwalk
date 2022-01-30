use std::{
    error::Error,
    fmt::{Debug, Display},
};

use bindings::Windows::Win32::System::Diagnostics::Debug::{GetLastError, WIN32_ERROR};

pub struct WindowsError(WIN32_ERROR);

impl WindowsError {
    pub fn last_error() -> Self {
        Self {
            0: unsafe { GetLastError() },
        }
    }
}

impl Display for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Debug for WindowsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for WindowsError {}
