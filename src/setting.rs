use std::path::{Path, PathBuf};

pub const PROJECT_LOCALES: &str = "locales";
pub const PROJECT_SRC: &str = "src";
pub const PROJECT_LIB: &str = "lib";
pub const PROJECT_SEARCH_ROOT: &str = "package.json";

pub fn get_project_lib() -> PathBuf {
    return Path::new(PROJECT_SRC).join(PROJECT_LIB);
}
