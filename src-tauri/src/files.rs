use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    domain::{SourceFileSummary, SourceFormat},
    error::{AppError, AppResult},
};

pub fn summarize_source(path: &Path) -> AppResult<SourceFileSummary> {
    let format = SourceFormat::from_path(path)?;
    let metadata = path.metadata()?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::MissingFileName(path.to_path_buf()))?
        .to_string();

    Ok(SourceFileSummary {
        path: path.to_string_lossy().into_owned(),
        file_name,
        format,
        size_bytes: metadata.len(),
        sha256: sha256_file(path)?,
    })
}

pub fn copy_source_to_workdir(source_path: &Path, workdir: &Path) -> AppResult<PathBuf> {
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::MissingFileName(source_path.to_path_buf()))?;
    let original_dir = workdir.join("original");
    std::fs::create_dir_all(&original_dir)?;
    let destination = original_dir.join(file_name);
    std::fs::copy(source_path, &destination)?;
    Ok(destination)
}

fn sha256_file(path: &Path) -> AppResult<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 64];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}
