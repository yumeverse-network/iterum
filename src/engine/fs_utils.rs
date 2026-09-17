use dirs;
use std::path::PathBuf;

pub enum DirType {
    Home,
    Desktop,
    Documents,
    Cache,
    Executables,
}

pub fn get_dir(dir_type: DirType) -> Option<PathBuf> {
    match dir_type {
        DirType::Home => dirs::home_dir(),
        DirType::Desktop => dirs::desktop_dir(),
        DirType::Documents => dirs::document_dir(),
        DirType::Cache => dirs::cache_dir(),
        DirType::Executables => dirs::executable_dir(),
    }
}
