use std::collections::HashMap;
use std::path::PathBuf;
use std::{collections::HashSet, error::Error, path::Path};

use bindings::Windows::Win32::Foundation::PSTR;
use bindings::Windows::Win32::System::SystemInformation::{
    GetSystemDirectoryA, GetWindowsDirectoryA,
};
use log::info;
use regex::Regex;

use crate::error::WindowsError;
use crate::registry::{RegistryKey, RootKey};
use crate::DllType;

#[derive(Debug)]
pub struct SearchPath {
    safe_search_enabled: bool,
    base_directory_files: HashMap<String, PathBuf>,
    known_dll_files: HashMap<String, PathBuf>,
    system_directory_files: HashMap<String, PathBuf>,
    windows_directory_files: HashMap<String, PathBuf>,
    path_directory_files: Vec<HashMap<String, PathBuf>>,
    current_directory_files: HashMap<String, PathBuf>,
    umbrella_dll_regex: Regex,
}

impl SearchPath {
    pub fn new(
        base_directory: &Path,
        current_directory: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let safe_search_enabled = SearchPath::safe_search_enabled();
        info!("Safe search enabled: {}", safe_search_enabled);

        let system_directory = SearchPath::get_system_directory()?;
        info!("System directory: {}", system_directory.to_string_lossy());
        info!("Base directory: {}", base_directory.to_string_lossy());
        info!("Current directory: {}", current_directory.to_string_lossy());

        let known_dll_files = SearchPath::get_knwon_dll_files()?
            .into_iter()
            .map(|name| (name.clone(), system_directory.join(name)))
            .collect();

        let base_directory_files = SearchPath::read_directory_files(base_directory)?;
        let system_directory_files = SearchPath::read_directory_files(&system_directory)?;

        let windows_directory = SearchPath::get_windows_directory()?;
        let windows_directory_files = SearchPath::read_directory_files(&windows_directory)?;

        let path_directories = SearchPath::get_path_directories();
        let mut path_directory_files = Vec::new();
        for directory in path_directories {
            match SearchPath::read_directory_files(&directory) {
                Ok(files) => path_directory_files.push(files),
                Err(_) => info!("Failed to read files in {:?}", &directory),
            }
        }

        let current_directory_files = SearchPath::read_directory_files(current_directory)?;

        Ok(SearchPath {
            safe_search_enabled,
            base_directory_files,
            known_dll_files,
            system_directory_files,
            windows_directory_files,
            path_directory_files,
            current_directory_files,
            umbrella_dll_regex: Regex::new(r"(api|ext)-.*-l\d+-\d+-\d+.dll").unwrap(),
        })
    }

    pub fn search(&self, name: &str) -> Option<(PathBuf, DllType)> {
        let name = name.to_lowercase();

        if self.safe_search_enabled {
            if let Some(path) = self.known_dll_files.get(&name) {
                return Some((path.to_owned(), DllType::Known));
            }

            if let Some(path) = self.base_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::User));
            }

            if let Some(path) = self.system_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::System));
            }

            if let Some(path) = self.windows_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::System));
            }

            if let Some(path) = self.current_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::User));
            }

            for files in &self.path_directory_files {
                if let Some(path) = files.get(&name) {
                    return Some((path.to_owned(), DllType::Path));
                }
            }

            if self.umbrella_dll_regex.is_match(&name) {
                return Some((PathBuf::new(), DllType::Umbrella));
            }

            None
        } else {
            if let Some(path) = self.known_dll_files.get(&name) {
                return Some((path.to_owned(), DllType::Known));
            }

            if let Some(path) = self.base_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::User));
            }

            if let Some(path) = self.current_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::User));
            }

            if let Some(path) = self.system_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::System));
            }

            if let Some(path) = self.windows_directory_files.get(&name) {
                return Some((path.to_owned(), DllType::System));
            }

            for files in &self.path_directory_files {
                if let Some(path) = files.get(&name) {
                    return Some((path.to_owned(), DllType::Path));
                }
            }

            if self.umbrella_dll_regex.is_match(&name) {
                return Some((PathBuf::new(), DllType::Umbrella));
            }

            None
        }
    }

    fn read_directory_files(path: &Path) -> Result<HashMap<String, PathBuf>, Box<dyn Error>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if !path.is_file() {
                    return None;
                }
                let name = path.file_name()?.to_str()?.to_lowercase();
                Some((name, path))
            })
            .collect::<HashMap<_, _>>())
    }

    pub fn get_system_directory() -> Result<PathBuf, Box<dyn Error>> {
        let mut buffer = vec![0u8; 256];
        let result = unsafe {
            GetSystemDirectoryA(
                PSTR {
                    0: buffer.as_mut_ptr(),
                },
                buffer.len() as u32,
            )
        };
        if result == 0 {
            Err(Box::new(WindowsError::last_error()))
        } else {
            Ok(PathBuf::from(
                std::str::from_utf8(&buffer)?.trim_end_matches('\x00'),
            ))
        }
    }

    fn get_windows_directory() -> Result<PathBuf, Box<dyn Error>> {
        let mut buffer = vec![0u8; 256];
        let result = unsafe {
            GetWindowsDirectoryA(
                PSTR {
                    0: buffer.as_mut_ptr(),
                },
                buffer.len() as u32,
            )
        };
        if result == 0 {
            Err(Box::new(WindowsError::last_error()))
        } else {
            Ok(PathBuf::from(
                std::str::from_utf8(&buffer)?.trim_end_matches('\x00'),
            ))
        }
    }

    fn get_path_directories() -> Vec<PathBuf> {
        //TODO Check if App Paths are included and remove them
        match std::env::var_os("PATH") {
            Some(paths) => std::env::split_paths(&paths).collect(),
            None => vec![],
        }
    }

    fn get_knwon_dll_files() -> Result<HashSet<String>, Box<dyn Error>> {
        let values = RegistryKey::root(RootKey::LocalMachine)
            .value_names(r"SYSTEM\CurrentControlSet\Control\Session Manager\KnownDLLs")?;

        let files = values
            .iter()
            .filter_map(|value| {
                RegistryKey::root(RootKey::LocalMachine)
                    .read_string(
                        r"SYSTEM\CurrentControlSet\Control\Session Manager\KnownDLLs",
                        value,
                    )
                    .ok()
            })
            .map(|name| name.to_lowercase())
            .collect();

        Ok(files)
    }

    fn safe_search_enabled() -> bool {
        let value = RegistryKey::root(RootKey::LocalMachine).read_dword(
            r"System\CurrentControlSet\Control\Session Manager",
            "SafeDllSearchMode",
        );

        match value {
            Ok(value) => value != 0,
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn search() {
        let cargo_dir = std::path::Path::new(env!("CARGO")).parent().unwrap();
        let search_path = SearchPath::new(cargo_dir, &PathBuf::new()).unwrap();

        assert_eq!(
            search_path.search("win32u.dll"),
            Some((
                PathBuf::from(r"C:\Windows\system32\win32u.dll"),
                DllType::System
            ))
        );

        assert_eq!(
            search_path.search("WIN32U.DLL"),
            Some((
                PathBuf::from(r"C:\Windows\system32\win32u.dll"),
                DllType::System
            ))
        );

        assert_eq!(
            search_path.search("cargo.exe"),
            Some((PathBuf::from(&cargo_dir.join("cargo.exe")), DllType::User))
        );

        assert_eq!(search_path.search("hopefully_not_existing.dll"), None);

        assert_eq!(
            search_path.search("api-ms-win-core-sysinfo-l1-2-3.dll"),
            Some((PathBuf::new(), DllType::Umbrella))
        );

        assert_eq!(
            search_path.search("kernel32.dll"),
            Some((PathBuf::from("C:\\Windows\\system32\\kernel32.dll"), DllType::Known))
        );
    }
}
