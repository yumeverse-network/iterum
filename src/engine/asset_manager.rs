use crate::engine::console::Console;

pub struct AssetManager {
    logger: Console,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager {
            logger: Console::new(),
        }
    }

    pub fn _mount_iia(&mut self, path: &str) {
        self.logger.log_system(format_args!("Mounted: {}", path));
    }

    pub fn _unmount_iia(&mut self, path: &str) {
        self.logger.log_system(format_args!("Unmounted: {}", path));
    }

    pub fn _load_asset(&mut self, path: &str) {
        self.logger.log_system(format_args!("Loaded: {}", path));
    }

    pub fn _unload_asset(&mut self, path: &str) {
        self.logger.log_system(format_args!("Unloaded: {}", path));
    }
}
