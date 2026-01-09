use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio::fs as async_fs;
use uuid::Uuid;
use crate::models::errors::AppError;

#[derive(Debug, Clone)]
pub struct FileStorageService {
    temp_dir: PathBuf,
    max_file_age: Duration,
}

impl FileStorageService {
    pub fn new(temp_dir: impl Into<PathBuf>) -> Result<Self, AppError> {
        let temp_dir = temp_dir.into();
        
        // Create the temporary directory if it doesn't exist
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir)
                .map_err(|e| AppError::storage_failed(format!("Failed to create temp directory: {}", e)))?;
        }

        Ok(Self {
            temp_dir,
            max_file_age: Duration::from_secs(3600), // 1 hour default
        })
    }

    /// Store a temporary file with UUID-based naming
    pub async fn store_temp_file(&self, data: &[u8], extension: &str) -> Result<String, AppError> {
        let file_id = Uuid::new_v4().to_string();
        let filename = format!("{}.{}", file_id, extension.trim_start_matches('.'));
        let file_path = self.temp_dir.join(&filename);

        async_fs::write(&file_path, data)
            .await
            .map_err(|e| AppError::storage_failed(format!("Failed to write temp file: {}", e)))?;

        tracing::debug!("Stored temporary file: {}", filename);
        Ok(file_id)
    }

    /// Get the full path for a temporary file
    pub fn get_temp_file_path(&self, file_id: &str, extension: &str) -> PathBuf {
        let filename = format!("{}.{}", file_id, extension.trim_start_matches('.'));
        self.temp_dir.join(filename)
    }

    /// Read a temporary file
    pub async fn read_temp_file(&self, file_id: &str, extension: &str) -> Result<Vec<u8>, AppError> {
        let file_path = self.get_temp_file_path(file_id, extension);
        
        if !file_path.exists() {
            return Err(AppError::storage_failed("Temporary file not found"));
        }

        async_fs::read(&file_path)
            .await
            .map_err(|e| AppError::storage_failed(format!("Failed to read temp file: {}", e)))
    }

    /// Delete a specific temporary file
    pub async fn delete_temp_file(&self, file_id: &str, extension: &str) -> Result<(), AppError> {
        let file_path = self.get_temp_file_path(file_id, extension);
        
        if file_path.exists() {
            async_fs::remove_file(&file_path)
                .await
                .map_err(|e| AppError::storage_failed(format!("Failed to delete temp file: {}", e)))?;
            
            tracing::debug!("Deleted temporary file: {}", file_path.display());
        }

        Ok(())
    }

    /// Clean up old temporary files
    pub async fn cleanup_temp_files(&self) -> Result<usize, AppError> {
        let mut cleaned_count = 0;
        let cutoff_time = SystemTime::now() - self.max_file_age;

        let mut entries = async_fs::read_dir(&self.temp_dir)
            .await
            .map_err(|e| AppError::storage_failed(format!("Failed to read temp directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::storage_failed(format!("Failed to read directory entry: {}", e)))? {
            
            let path = entry.path();
            
            if path.is_file() {
                let metadata = entry.metadata().await
                    .map_err(|e| AppError::storage_failed(format!("Failed to read file metadata: {}", e)))?;
                
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        if let Err(e) = async_fs::remove_file(&path).await {
                            tracing::warn!("Failed to delete old temp file {}: {}", path.display(), e);
                        } else {
                            cleaned_count += 1;
                            tracing::debug!("Cleaned up old temp file: {}", path.display());
                        }
                    }
                }
            }
        }

        if cleaned_count > 0 {
            tracing::info!("Cleaned up {} old temporary files", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Get the size of a temporary file
    pub async fn get_file_size(&self, file_id: &str, extension: &str) -> Result<u64, AppError> {
        let file_path = self.get_temp_file_path(file_id, extension);
        
        let metadata = async_fs::metadata(&file_path)
            .await
            .map_err(|e| AppError::storage_failed(format!("Failed to get file metadata: {}", e)))?;
        
        Ok(metadata.len())
    }

    /// Check if a temporary file exists
    pub fn temp_file_exists(&self, file_id: &str, extension: &str) -> bool {
        self.get_temp_file_path(file_id, extension).exists()
    }

    /// Set the maximum age for temporary files
    pub fn set_max_file_age(&mut self, max_age: Duration) {
        self.max_file_age = max_age;
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }
}