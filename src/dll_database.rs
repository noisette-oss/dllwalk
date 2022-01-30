use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use log::{debug, error, info};

use crate::{pe::File, search_path::SearchPath, DllType};

#[derive(Debug)]
pub struct DllInfo {
    pub path: PathBuf,
    pub dll_type: DllType,
    pub file: File,
}

pub struct DllDatabase {
    files: HashMap<String, Option<DllInfo>>,
    search_path: SearchPath,
}

impl DllDatabase {
    pub fn new(base_directory: &Path, current_directory: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            files: HashMap::new(),
            search_path: SearchPath::new(base_directory, current_directory)?,
        })
    }

    pub fn get_dll_info(&self, name: &str) -> Option<&DllInfo> {
        if let Some(Some(info)) = self.files.get(name) {
            return Some(info);
        }

        None
    }

    pub fn search_dll(&mut self, name: &str) -> Option<&DllInfo> {
        if self.get_dll_info(name).is_none() {
            debug!("Searching for {}", name);

            let info = match self.search_path.search(name) {
                Some((path, dll_type)) => {
                    let path_str = path.to_string_lossy();
                    info!(
                        "Found {} ({})",
                        if path_str.is_empty() { name } else { &path_str },
                        dll_type
                    );
                    DllDatabase::parse_dll(path, dll_type)
                }
                None => {
                    error!("Could not find {}", name);
                    None
                }
            };

            self.files.insert(name.to_string(), info);
        }

        self.get_dll_info(name)
    }

    pub fn get_all_dlls(&self) -> Vec<String> {
        return self.files.keys().map(|key| key.to_owned()).collect::<_>();
    }

    fn parse_dll(path: PathBuf, dll_type: DllType) -> Option<DllInfo> {
        if dll_type == DllType::Umbrella {
            return Some(DllInfo {
                path,
                dll_type,
                file: File::new(),
            });
        }

        debug!("Parsing {}", path.to_string_lossy());
        match std::fs::read(&path) {
            Ok(data) => match File::parse(&data) {
                Ok((_, file)) => Some(DllInfo {
                    path,
                    dll_type,
                    file,
                }),
                Err(err) => {
                    error!("Failed to parse {}: {}", path.to_string_lossy(), err);
                    None
                }
            },
            Err(err) => {
                error!("Failed to read {}: {}", path.to_string_lossy(), err);
                None
            }
        }
    }
}
