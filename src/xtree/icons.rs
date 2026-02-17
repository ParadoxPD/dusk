use std::path::Path;

pub use crate::core::devicons::{ICON_DIR, ICON_EXEC, ICON_LINK};

pub fn file_icon(path: &Path) -> &'static str {
    crate::core::devicons::file_icon(path)
}
