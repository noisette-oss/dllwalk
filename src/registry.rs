use std::collections::HashSet;

use bindings::Windows::Win32::{
    Foundation::PWSTR,
    System::{
        Diagnostics::Debug::{ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
        Registry::{
            RegCloseKey, RegEnumValueW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CLASSES_ROOT,
            HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_READ,
            RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
        },
    },
};

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootKey {
    LocalMachine,
    CurrentConfig,
    ClassesRoot,
    CurrentUser,
    Users,
}

impl From<RootKey> for HKEY {
    fn from(key: RootKey) -> Self {
        match key {
            RootKey::LocalMachine => HKEY_LOCAL_MACHINE,
            RootKey::CurrentConfig => HKEY_CURRENT_CONFIG,
            RootKey::ClassesRoot => HKEY_CLASSES_ROOT,
            RootKey::CurrentUser => HKEY_CURRENT_USER,
            RootKey::Users => HKEY_USERS,
        }
    }
}

struct ErrorCode;

impl ErrorCode {
    const ERROR_SUCCESS: i32 = ERROR_SUCCESS.0 as _;
    const ERROR_NO_MORE_ITEMS: i32 = ERROR_NO_MORE_ITEMS.0 as _;
    const ERROR_MORE_DATA: i32 = ERROR_MORE_DATA.0 as _;
}

#[derive(Clone)]
pub struct RegistryError {
    message: String,
}

impl RegistryError {
    fn new<S: ToString>(message: S) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::error::Error for RegistryError {}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "RegistryError: {}", self.message)
    }
}

impl std::fmt::Debug for RegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "RegistryError: {}", self.message)
    }
}

pub struct RegistryKey {
    handle: HKEY,
}

impl RegistryKey {
    pub fn root(root: RootKey) -> Self {
        Self {
            handle: root.into(),
        }
    }

    pub fn value_names(&self, subkey: &str) -> Result<HashSet<String>, RegistryError> {
        // Open the key
        let mut handle = HKEY::NULL;
        let error_code = unsafe { RegOpenKeyExW(self.handle, subkey, 0, KEY_READ, &mut handle) };

        if error_code.0 != ErrorCode::ERROR_SUCCESS {
            return Err(RegistryError::new(format!(
                "Failed to open key: {}",
                subkey
            )));
        }

        // Loop over values
        let mut names = HashSet::new();

        let mut buffer = vec![0u16; 256];
        let mut index = 0;

        loop {
            let mut size = buffer.len() as u32;

            let error_code = unsafe {
                RegEnumValueW(
                    handle,
                    index,
                    PWSTR(buffer.as_mut_ptr()),
                    &mut size,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };

            match error_code.0 {
                ErrorCode::ERROR_SUCCESS => {
                    let string = String::from_utf16_lossy(&buffer[..size as _]);
                    names.insert(string);
                    index += 1;
                }
                ErrorCode::ERROR_MORE_DATA => {
                    // Double buffer size
                    buffer = vec![0; 2 * buffer.len()];
                }
                ErrorCode::ERROR_NO_MORE_ITEMS => {
                    return Ok(names);
                }
                _ => {
                    return Err(RegistryError::new(format!(
                        "Failed to enumerate values of key: {}",
                        subkey
                    )));
                }
            }
        }
    }

    pub fn read_string(&self, subkey: &str, value_name: &str) -> Result<String, RegistryError> {
        let mut size = 0;
        let error_code = unsafe {
            RegGetValueW(
                self.handle,
                subkey,
                value_name,
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };

        if error_code.0 != ErrorCode::ERROR_SUCCESS {
            return Err(RegistryError::new(format!(
                "Failed to read value: {}\\{}",
                subkey, value_name
            )));
        }

        let mut buffer = vec![0u16; size as usize];

        let error_code = unsafe {
            RegGetValueW(
                self.handle,
                subkey,
                value_name,
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as _,
                &mut size,
            )
        };

        if error_code.0 != ErrorCode::ERROR_SUCCESS {
            return Err(RegistryError::new(format!(
                "Failed to read value: {}\\{}",
                subkey, value_name
            )));
        }

        let value = String::from_utf16_lossy(&buffer)
            .trim_end_matches(|c| c == '\0')
            .to_owned();

        Ok(value)
    }

    pub fn read_dword(&self, subkey: &str, value_name: &str) -> Result<u32, RegistryError> {
        let mut size = 4;
        let mut value = 0u32;
        let error_code = unsafe {
            RegGetValueW(
                self.handle,
                subkey,
                value_name,
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                &mut value as *mut u32 as _,
                &mut size,
            )
        };

        if error_code.0 != ErrorCode::ERROR_SUCCESS {
            return Err(RegistryError::new(format!(
                "Failed to read value: {}\\{}",
                subkey, value_name
            )));
        }

        Ok(value)
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.handle);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_dword() {
        let value = RegistryKey::root(RootKey::LocalMachine).read_dword(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\",
            "CurrentMajorVersionNumber",
        );

        assert_eq!(value.is_ok(), true);

        let value = RegistryKey::root(RootKey::LocalMachine).read_dword(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
        assert_eq!(value.is_err(), true);
    }

    #[test]
    fn read_string() {
        let value = RegistryKey::root(RootKey::LocalMachine).read_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\",
            "SystemRoot",
        );

        assert_eq!(value.is_ok(), true);

        let value = RegistryKey::root(RootKey::LocalMachine).read_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\",
            "invalid_value",
        );
        assert_eq!(value.is_err(), true);
    }

    #[test]
    fn value_names() {
        let names = RegistryKey::root(RootKey::LocalMachine)
            .value_names(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");

        assert_eq!(names.is_ok(), true);
        let names = names.unwrap();
        assert_eq!(names.contains("RegisteredOwner"), true);
        assert_eq!(names.contains("SystemRoot"), true);

        let names = RegistryKey::root(RootKey::LocalMachine)
            .value_names(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\invalid_value");
        assert_eq!(names.is_err(), true);
    }
}
