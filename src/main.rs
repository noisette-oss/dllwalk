use std::path::PathBuf;

use crate::dll_database::DllDatabase;

mod dll_database;
mod error;
mod pe;
mod registry;
mod search_path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DllType {
    User,
    Path,
    System,
    Known,
    Umbrella,
}

impl std::fmt::Display for DllType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DllType::User => write!(formatter, "user-dll"),
            DllType::Path => write!(formatter, "path-dll"),
            DllType::System => write!(formatter, "system-dll"),
            DllType::Known => write!(formatter, "known-dll"),
            DllType::Umbrella => write!(formatter, "umbrella-dll"),
        }
    }
}

fn walk_dlls(database: &mut DllDatabase, name: &str) {
    let mut visited = std::collections::HashSet::new();
    let mut queue = Vec::new();
    queue.push(name.to_owned());

    while !queue.is_empty() {
        let name = queue.pop().unwrap();

        if let Some(info) =  database.search_dll(&name) {
            for dll in &info.file.imports {
                if !visited.contains(&dll.name) {
                    queue.push(dll.name.clone());
                }
            }
        }

        visited.insert(name);
    }
}


use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print the imported dlls as a tree
    Tree {
        /// File to parse
        file: PathBuf,

        /// Show the files absolute path
        #[clap(short, long)]
        absolute_path: bool,

        // Maximum depth
        #[clap(short, long)]
        depth: Option<u32>,
    },

    /// List the imported dlls
    List { 
        /// File to parse
        file: PathBuf,

        /// Show the files absolute path
        #[clap(short, long)]
        absolute_path: bool,
    },
}


pub struct TreePrinter {
    max_depth: Option<u32>,
    absolute_path: bool,
}

impl TreePrinter {
    pub fn new(max_depth: Option<u32>, absolute_path: bool) -> Self {
        Self {
            max_depth,
            absolute_path,
        }
    }

    pub fn print(&self, database: &DllDatabase, name: &str, depth: u32, last_child: bool) {
        TreePrinter::print_prefix(depth, last_child);
        
        if self.absolute_path {
            if let Some(info) = database.get_dll_info(name)  {
                let path = info.path.to_string_lossy();
                println!("{}", if path.is_empty() { name } else { &path });
            }
        }
        else {
            println!("{}", name);
        }
        
    
        if let Some(info) = database.get_dll_info(name) {
            for (index, dll) in info.file.imports.iter().enumerate() {
                if depth < self.max_depth.unwrap_or(u32::MAX) {
                    self.print(database, &dll.name, depth + 1, index ==  info.file.imports.len() - 1);
                }
            }
        }
    }

    fn print_prefix(depth: u32, last_child: bool) {
        if depth > 1 {
            for _ in 0..depth - 1 {
                print!("│   ");
            }
        }
        if depth > 0 {   
            if last_child {
                print!("└── ");
            }
            else {
                print!("├── ");
            }
        }
    }
}

fn print_list(database: &DllDatabase, absolute_path: bool) {
    let dlls = database.get_all_dlls();
    for dll in dlls {
        if absolute_path {
            if let Some(info) = database.get_dll_info(&dll)  {
                let path = info.path.to_string_lossy().to_string();
                println!("{}", if path.is_empty() { &dll } else { &path });
            }
        }
        else {
            println!("{}", dll);
        }
    }
}


fn main() {
    env_logger::init();

    let args = Arguments::parse();

    let current_directory = std::env::current_dir().expect("Failed to get current directory");

    let file = match &args.command {
        Commands::Tree { file, ..} => file,
        Commands::List { file, ..} => file,
    };

    let base_directory = file.parent().unwrap_or(&current_directory);

    let mut database = DllDatabase::new(base_directory, &current_directory)
        .expect("Failed to initialize the dll database");

    let file = file.file_name().unwrap().to_string_lossy();

    walk_dlls(&mut database, &file);

    match args.command {
        Commands::Tree { absolute_path, depth , ..} => {
            let printer = TreePrinter::new(depth, absolute_path);
            printer.print(&database, &file, 0, false);
        },
        Commands::List { absolute_path , ..} => {
            print_list(&database, absolute_path);
        },
    }
}
