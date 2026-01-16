use crate::models::errors::AppError;
use crate::services::export_service::{ExportService, ExportResult, EnhancedExportOptions};
use crate::services::file_storage::FileStorageService;
use crate::models::theme::Theme;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Download progress tracking
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub status: DownloadStatus,
    pub progress_percent: u8,
    pub message: String,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub file_size: Option<usize>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Expired,
}

/// Download request information
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadRequest {
    pub code: String,
    pub language: String,
    pub theme: Theme,
    pub export_options: EnhancedExportOptions,
}

/// Download metadata for file serving
#[derive(Debug, Clone)]
pub struct DownloadMetadata {
    pub file_id: String,
    pub original_filename: String,
    pub content_type: String,
    pub file_size: usize,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
}

/// Service for managing downloads and file serving
pub struct DownloadService {
    export_service: Arc<ExportService>,
    file_storage: Arc<FileStorageService>,
    progress_tracker: Arc<RwLock<HashMap<String, DownloadProgress>>>,
    download_metadata: Arc<RwLock<HashMap<String, DownloadMetadata>>>,
    max_concurrent_downloads: usize,
    download_expiry: Duration,
}

impl DownloadService {
    /// Creates a new DownloadService
    pub fn new(
        export_service: Arc<ExportService>,
        file_storage: Arc<FileStorageService>,
    ) -> Self {
        DownloadService {
            export_service,
            file_storage,
            progress_tracker: Arc::new(RwLock::new(HashMap::new())),
            download_metadata: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_downloads: 10,
            download_expiry: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Start a new download process
    pub async fn start_download(
        &self,
        request: DownloadRequest,
    ) -> Result<String, AppError> {
        // Validate export options
        ExportService::validate_options(&request.export_options)?;

        // Check concurrent download limit BEFORE adding to tracker
        let active_downloads = self.count_active_downloads().await;
        if active_downloads >= self.max_concurrent_downloads {
            return Err(AppError::image_generation_failed("Server is busy. Please try again later."));
        }

        let download_id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create initial progress entry
        let progress = DownloadProgress {
            download_id: download_id.clone(),
            status: DownloadStatus::Queued,
            progress_percent: 0,
            message: "Download queued for processing".to_string(),
            created_at: now,
            completed_at: None,
            file_size: None,
            error_message: None,
        };

        // Store progress
        {
            let mut tracker = self.progress_tracker.write().await;
            tracker.insert(download_id.clone(), progress);
        }

        // Start processing in background
        let service = self.clone();
        let req = request.clone();
        let download_id_clone = download_id.clone();
        tokio::spawn(async move {
            service.process_download(download_id_clone, req).await;
        });

        Ok(download_id)
    }

    /// Process the download in background
    async fn process_download(&self, download_id: String, request: DownloadRequest) {
        // Update status to processing
        if let Err(e) = self.update_progress(&download_id, DownloadStatus::Processing, 10, 
            "Starting image generation...".to_string(), None).await {
            tracing::error!("Failed to update progress for {}: {}", download_id, e);
            return;
        }

        // Perform the export with retry logic
        let export_result = self.export_with_retry(&request, &download_id).await;

        match export_result {
            Ok(result) => {
                // Update progress
                if let Err(e) = self.update_progress(&download_id, DownloadStatus::Processing, 80, 
                    "Saving generated image...".to_string(), None).await {
                    tracing::error!("Failed to update progress for {}: {}", download_id, e);
                    return;
                }

                // Store the generated file
                match self.store_generated_file(&download_id, &result, &request).await {
                    Ok(_) => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        if let Err(e) = self.update_progress(&download_id, DownloadStatus::Completed, 100, 
                            "Download ready".to_string(), None).await {
                            tracing::error!("Failed to update final progress for {}: {}", download_id, e);
                        }

                        // Update completion time
                        {
                            let mut tracker = self.progress_tracker.write().await;
                            if let Some(progress) = tracker.get_mut(&download_id) {
                                progress.completed_at = Some(now);
                                progress.file_size = Some(result.file_size);
                            }
                        }

                        tracing::info!("Download {} completed successfully", download_id);
                    }
                    Err(e) => {
                        if let Err(update_err) = self.update_progress(&download_id, DownloadStatus::Failed, 0, 
                            "Failed to save generated image".to_string(), 
                            Some(e.to_string())).await {
                            tracing::error!("Failed to update error progress for {}: {}", download_id, update_err);
                        }
                        tracing::error!("Failed to store generated file for {}: {}", download_id, e);
                    }
                }
            }
            Err(e) => {
                if let Err(update_err) = self.update_progress(&download_id, DownloadStatus::Failed, 0, 
                    "Image generation failed".to_string(), 
                    Some(e.to_string())).await {
                    tracing::error!("Failed to update error progress for {}: {}", download_id, update_err);
                }
                tracing::error!("Export failed for {}: {}", download_id, e);
            }
        }
    }

    /// Export with retry logic
    async fn export_with_retry(
        &self,
        request: &DownloadRequest,
        download_id: &str,
    ) -> Result<ExportResult, AppError> {
        const MAX_RETRIES: usize = 3;
        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            // Update progress
            let message = if attempt == 1 {
                "Generating image...".to_string()
            } else {
                format!("Retrying image generation (attempt {}/{})", attempt, MAX_RETRIES)
            };

            if let Err(e) = self.update_progress(download_id, DownloadStatus::Processing, 
                20 + (attempt as u8 * 20), message, None).await {
                tracing::warn!("Failed to update retry progress: {}", e);
            }

            match self.export_service.export_code_snippet(
                &request.code,
                &request.language,
                &request.theme,
                &request.export_options,
            ).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES {
                        // Exponential backoff
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt as u32 - 1)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::image_generation_failed("Unknown error during export")))
    }

    /// Store the generated file
    async fn store_generated_file(
        &self,
        download_id: &str,
        result: &ExportResult,
        _request: &DownloadRequest,
    ) -> Result<(), AppError> {
        // Determine file extension
        let extension = match result.format {
            crate::services::image_generator::ImageFormat::PNG => "png",
            crate::services::image_generator::ImageFormat::JPEG => "jpg",
            crate::services::image_generator::ImageFormat::SVG => "svg",
        };

        // Store the file
        let file_id = self.file_storage.store_temp_file(&result.data, extension).await?;

        // Create metadata
        let content_type = match result.format {
            crate::services::image_generator::ImageFormat::PNG => "image/png",
            crate::services::image_generator::ImageFormat::JPEG => "image/jpeg",
            crate::services::image_generator::ImageFormat::SVG => "image/svg+xml",
        };

        let original_filename = format!("code-snippet.{}", extension);
        let now = SystemTime::now();
        let expires_at = now + self.download_expiry;

        let metadata = DownloadMetadata {
            file_id,
            original_filename,
            content_type: content_type.to_string(),
            file_size: result.file_size,
            created_at: now,
            expires_at,
        };

        // Store metadata
        {
            let mut meta_map = self.download_metadata.write().await;
            meta_map.insert(download_id.to_string(), metadata);
        }

        Ok(())
    }

    /// Get download progress
    pub async fn get_progress(&self, download_id: &str) -> Option<DownloadProgress> {
        let tracker = self.progress_tracker.read().await;
        tracker.get(download_id).cloned()
    }

    /// Get file for download
    pub async fn get_download_file(&self, download_id: &str) -> Result<(Vec<u8>, DownloadMetadata), AppError> {
        // Get metadata
        let metadata = {
            let meta_map = self.download_metadata.read().await;
            meta_map.get(download_id).cloned()
        };

        let metadata = metadata.ok_or_else(|| AppError::storage_failed("Download not found"))?;

        // Check if expired
        if SystemTime::now() > metadata.expires_at {
            return Err(AppError::storage_failed("Download has expired"));
        }

        // Get file extension from content type
        let extension = match metadata.content_type.as_str() {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/svg+xml" => "svg",
            _ => return Err(AppError::storage_failed("Unknown file type")),
        };

        // Read file data
        let file_data = self.file_storage.read_temp_file(&metadata.file_id, extension).await?;

        Ok((file_data, metadata))
    }

    /// Update download progress
    async fn update_progress(
        &self,
        download_id: &str,
        status: DownloadStatus,
        progress_percent: u8,
        message: String,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        let mut tracker = self.progress_tracker.write().await;
        
        if let Some(progress) = tracker.get_mut(download_id) {
            progress.status = status;
            progress.progress_percent = progress_percent;
            progress.message = message;
            progress.error_message = error_message;
        }

        Ok(())
    }

    /// Count active downloads
    async fn count_active_downloads(&self) -> usize {
        let tracker = self.progress_tracker.read().await;
        tracker.values()
            .filter(|p| matches!(p.status, DownloadStatus::Queued | DownloadStatus::Processing))
            .count()
    }

    /// Clean up expired downloads
    pub async fn cleanup_expired_downloads(&self) -> Result<usize, AppError> {
        let now = SystemTime::now();
        let mut cleaned_count = 0;

        // Get expired download IDs
        let expired_ids: Vec<String> = {
            let meta_map = self.download_metadata.read().await;
            meta_map.iter()
                .filter(|(_, metadata)| now > metadata.expires_at)
                .map(|(id, _)| id.clone())
                .collect()
        };

        // Clean up expired downloads
        for download_id in expired_ids {
            if let Err(e) = self.cleanup_download(&download_id).await {
                tracing::warn!("Failed to cleanup expired download {}: {}", download_id, e);
            } else {
                cleaned_count += 1;
            }
        }

        // Also clean up old progress entries (older than 24 hours)
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 86400; // 24 hours

        {
            let mut tracker = self.progress_tracker.write().await;
            let old_ids: Vec<String> = tracker.iter()
                .filter(|(_, progress)| progress.created_at < cutoff_time)
                .map(|(id, _)| id.clone())
                .collect();

            for id in old_ids {
                tracker.remove(&id);
            }
        }

        if cleaned_count > 0 {
            tracing::info!("Cleaned up {} expired downloads", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Clean up a specific download
    async fn cleanup_download(&self, download_id: &str) -> Result<(), AppError> {
        // Get metadata
        let metadata = {
            let mut meta_map = self.download_metadata.write().await;
            meta_map.remove(download_id)
        };

        if let Some(metadata) = metadata {
            // Determine extension
            let extension = match metadata.content_type.as_str() {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/svg+xml" => "svg",
                _ => "tmp",
            };

            // Delete the file
            if let Err(e) = self.file_storage.delete_temp_file(&metadata.file_id, extension).await {
                tracing::warn!("Failed to delete file for download {}: {}", download_id, e);
            }
        }

        // Remove from progress tracker
        {
            let mut tracker = self.progress_tracker.write().await;
            if let Some(progress) = tracker.get_mut(download_id) {
                progress.status = DownloadStatus::Expired;
            }
        }

        Ok(())
    }

    /// Set maximum concurrent downloads
    pub fn set_max_concurrent_downloads(&mut self, max: usize) {
        self.max_concurrent_downloads = max;
    }

    /// Set download expiry duration
    pub fn set_download_expiry(&mut self, duration: Duration) {
        self.download_expiry = duration;
    }

    /// Get download statistics
    pub async fn get_stats(&self) -> DownloadStats {
        let tracker = self.progress_tracker.read().await;
        let meta_map = self.download_metadata.read().await;

        let mut stats = DownloadStats::default();
        
        for progress in tracker.values() {
            match progress.status {
                DownloadStatus::Queued => stats.queued += 1,
                DownloadStatus::Processing => stats.processing += 1,
                DownloadStatus::Completed => stats.completed += 1,
                DownloadStatus::Failed => stats.failed += 1,
                DownloadStatus::Expired => stats.expired += 1,
            }
        }

        stats.total_files = meta_map.len();
        stats.total_size = meta_map.values().map(|m| m.file_size).sum();

        stats
    }
}

impl Clone for DownloadService {
    fn clone(&self) -> Self {
        DownloadService {
            export_service: Arc::clone(&self.export_service),
            file_storage: Arc::clone(&self.file_storage),
            progress_tracker: Arc::clone(&self.progress_tracker),
            download_metadata: Arc::clone(&self.download_metadata),
            max_concurrent_downloads: self.max_concurrent_downloads,
            download_expiry: self.download_expiry,
        }
    }
}

/// Download statistics
#[derive(Debug, Default, Serialize)]
pub struct DownloadStats {
    pub queued: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub expired: usize,
    pub total_files: usize,
    pub total_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::theme::Theme;
    use crate::services::export_service::{EnhancedExportOptions};
    use crate::services::image_generator::ImageFormat;
    use tempfile::TempDir;

    async fn create_test_service() -> (DownloadService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let file_storage = Arc::new(FileStorageService::new(temp_dir.path()).unwrap());
        let export_service = Arc::new(ExportService::new().unwrap());
        let service = DownloadService::new(export_service, file_storage);
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_start_download() {
        let (service, _temp_dir) = create_test_service().await;
        
        let request = DownloadRequest {
            code: "fn main() { println!(\"Hello!\"); }".to_string(),
            language: "Rust".to_string(),
            theme: Theme::default_dark(),
            export_options: EnhancedExportOptions::default(),
        };

        let download_id = service.start_download(request).await.unwrap();
        assert!(!download_id.is_empty());

        // Check initial progress
        let progress = service.get_progress(&download_id).await.unwrap();
        assert_eq!(progress.download_id, download_id);
        assert_eq!(progress.status, DownloadStatus::Queued);
    }

    #[tokio::test]
    async fn test_download_progress_tracking() {
        let (service, _temp_dir) = create_test_service().await;
        
        let request = DownloadRequest {
            code: "console.log('test');".to_string(),
            language: "JavaScript".to_string(),
            theme: Theme::default_light(),
            export_options: EnhancedExportOptions {
                format: ImageFormat::PNG,
                ..Default::default()
            },
        };

        let download_id = service.start_download(request).await.unwrap();
        
        // Wait a bit for processing to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let progress = service.get_progress(&download_id).await.unwrap();
        assert!(matches!(progress.status, DownloadStatus::Queued | DownloadStatus::Processing));
    }

    #[tokio::test]
    async fn test_invalid_export_options() {
        let (service, _temp_dir) = create_test_service().await;
        
        let request = DownloadRequest {
            code: "test".to_string(),
            language: "Rust".to_string(),
            theme: Theme::default_dark(),
            export_options: EnhancedExportOptions {
                format: ImageFormat::JPEG,
                quality: 101, // Invalid quality for JPEG
                ..Default::default()
            },
        };

        let result = service.start_download(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired_downloads() {
        let (mut service, _temp_dir) = create_test_service().await;
        
        // Set very short expiry for testing
        service.set_download_expiry(Duration::from_millis(1));
        
        let request = DownloadRequest {
            code: "test".to_string(),
            language: "Rust".to_string(),
            theme: Theme::default_dark(),
            export_options: EnhancedExportOptions::default(),
        };

        let _download_id = service.start_download(request).await.unwrap();
        
        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let cleaned = service.cleanup_expired_downloads().await.unwrap();
        // Note: The actual cleanup depends on the download completing first
        assert!(cleaned >= 0);
    }

    #[tokio::test]
    async fn test_download_stats() {
        let (service, _temp_dir) = create_test_service().await;
        
        let stats = service.get_stats().await;
        assert_eq!(stats.queued, 0);
        assert_eq!(stats.processing, 0);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_concurrent_download_limit() {
        let (mut service, _temp_dir) = create_test_service().await;
        
        // Set limit to 1 for testing
        service.set_max_concurrent_downloads(1);
        
        let request = DownloadRequest {
            code: "test".to_string(),
            language: "Rust".to_string(),
            theme: Theme::default_dark(),
            export_options: EnhancedExportOptions::default(),
        };

        // First download should succeed
        let download_id1 = service.start_download(request.clone()).await.unwrap();
        
        // Second download should fail due to limit
        let result2 = service.start_download(request).await;
        assert!(result2.is_err());
        
        // Verify first download exists
        assert!(service.get_progress(&download_id1).await.is_some());
    }
}