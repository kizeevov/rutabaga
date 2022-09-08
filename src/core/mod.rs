use rfd::FileHandle;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ClearProcessError {
    kind: ErrorKind,
}

pub enum ErrorKind {
    ScanFolderError,
}

impl ClearProcessError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

pub async fn select_folder() -> Option<PathBuf> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("Folder selection")
        .pick_folder()
        .await;

    path.map_or(None, |f| Some(f.path().to_path_buf()))
}

pub async fn rename_and_clear_files_in_folder(path: &str) -> Result<(), ClearProcessError> {
    let paths = tokio::fs::read_dir(path)
        .await
        .map_err(|err| ClearProcessError::new(ErrorKind::ScanFolderError))?;

    Ok(())
}

pub async fn count_file_in_folder(path: PathBuf) -> usize {
    if let Ok(paths) = fs::read_dir(path) {
        paths
            .filter(|p| match p {
                Ok(f) => f.path().is_file(),
                Err(_) => false,
            })
            .count()
    } else {
        0
    }
}
