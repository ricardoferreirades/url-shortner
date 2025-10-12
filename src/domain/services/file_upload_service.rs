use image::ImageFormat;
use std::path::Path;
use thiserror::Error;

/// File upload service for handling profile pictures
pub struct FileUploadService {
    upload_dir: String,
    max_file_size: usize,
    allowed_extensions: Vec<String>,
    allowed_mime_types: Vec<String>,
}

/// File upload errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum FileUploadError {
    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(usize, usize),

    #[error("Invalid file type: {0} (allowed: {1:?})")]
    InvalidFileType(String, Vec<String>),

    #[error("Invalid image format")]
    InvalidImageFormat,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("File name too long: {0} characters (max: 255)")]
    FileNameTooLong(usize),

    #[error("Invalid file name: {0}")]
    InvalidFileName(String),
}

/// File upload result
#[derive(Debug, Clone)]
pub struct FileUploadResult {
    pub filename: String,
    #[allow(dead_code)]
    pub file_path: String,
    pub file_size: usize,
    #[allow(dead_code)]
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl FileUploadService {
    /// Create a new file upload service
    pub fn new(
        upload_dir: String,
        max_file_size: usize,
        allowed_extensions: Vec<String>,
        allowed_mime_types: Vec<String>,
    ) -> Self {
        Self {
            upload_dir,
            max_file_size,
            allowed_extensions,
            allowed_mime_types,
        }
    }

    /// Create a default file upload service for profile pictures
    pub fn new_profile_picture_service(upload_dir: String) -> Self {
        Self::new(
            upload_dir,
            5 * 1024 * 1024, // 5MB max file size
            vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
            vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
            ],
        )
    }

    /// Validate file before upload
    pub fn validate_file(
        &self,
        filename: &str,
        content_type: &str,
        file_size: usize,
    ) -> Result<(), FileUploadError> {
        // Check file size
        if file_size > self.max_file_size {
            return Err(FileUploadError::FileTooLarge(file_size, self.max_file_size));
        }

        // Check filename length
        if filename.len() > 255 {
            return Err(FileUploadError::FileNameTooLong(filename.len()));
        }

        // Check filename for invalid characters
        if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
            return Err(FileUploadError::InvalidFileName(
                "Filename contains invalid characters".to_string(),
            ));
        }

        // Check file extension
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| FileUploadError::InvalidFileName("No file extension".to_string()))?;

        if !self.allowed_extensions.contains(&extension) {
            return Err(FileUploadError::InvalidFileType(
                extension,
                self.allowed_extensions.clone(),
            ));
        }

        // Check MIME type
        if !self.allowed_mime_types.contains(&content_type.to_string()) {
            return Err(FileUploadError::InvalidFileType(
                content_type.to_string(),
                self.allowed_mime_types.clone(),
            ));
        }

        Ok(())
    }

    /// Process and save uploaded file
    pub async fn process_and_save_file(
        &self,
        filename: &str,
        content_type: &str,
        file_data: Vec<u8>,
    ) -> Result<FileUploadResult, FileUploadError> {
        // Validate file
        self.validate_file(filename, content_type, file_data.len())?;

        // Create upload directory if it doesn't exist
        tokio::fs::create_dir_all(&self.upload_dir).await?;

        // Generate unique filename
        let file_extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg");
        let unique_filename = format!("{}.{}", uuid::Uuid::new_v4(), file_extension);
        let file_path = Path::new(&self.upload_dir).join(&unique_filename);

        // Process image if it's an image file
        let file_size = file_data.len();
        let (processed_data, width, height) = if content_type.starts_with("image/") {
            self.process_image(file_data)?
        } else {
            (file_data, None, None)
        };

        // Save file
        tokio::fs::write(&file_path, processed_data).await?;

        Ok(FileUploadResult {
            filename: unique_filename,
            file_path: file_path.to_string_lossy().to_string(),
            file_size,
            mime_type: content_type.to_string(),
            width,
            height,
        })
    }

    /// Process image (resize, optimize, etc.)
    fn process_image(
        &self,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, Option<u32>, Option<u32>), FileUploadError> {
        let img = image::load_from_memory(&data)?;
        let (width, height) = (img.width(), img.height());

        // Resize image if it's too large (max 1024x1024)
        let processed_img = if width > 1024 || height > 1024 {
            let aspect_ratio = width as f32 / height as f32;
            let (new_width, new_height) = if aspect_ratio > 1.0 {
                (1024, (1024.0 / aspect_ratio) as u32)
            } else {
                ((1024.0 * aspect_ratio) as u32, 1024)
            };
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        // Convert to JPEG format for consistency
        let mut output = Vec::new();
        processed_img.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Jpeg)?;

        Ok((
            output,
            Some(processed_img.width()),
            Some(processed_img.height()),
        ))
    }

    /// Delete file
    pub async fn delete_file(&self, filename: &str) -> Result<(), FileUploadError> {
        let file_path = Path::new(&self.upload_dir).join(filename);
        if file_path.exists() {
            tokio::fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    /// Get file URL (for serving files)
    pub fn get_file_url(&self, filename: &str, base_url: &str) -> String {
        format!("{}/uploads/{}", base_url, filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_file() {
        let service = FileUploadService::new_profile_picture_service("/tmp".to_string());

        // Valid file
        assert!(service
            .validate_file("test.jpg", "image/jpeg", 1024)
            .is_ok());

        // File too large
        assert!(service
            .validate_file("test.jpg", "image/jpeg", 10 * 1024 * 1024)
            .is_err());

        // Invalid extension
        assert!(service
            .validate_file("test.txt", "text/plain", 1024)
            .is_err());

        // Invalid MIME type
        assert!(service
            .validate_file("test.jpg", "text/plain", 1024)
            .is_err());

        // Filename too long
        let long_name = "a".repeat(300);
        assert!(service
            .validate_file(&long_name, "image/jpeg", 1024)
            .is_err());

        // Invalid filename
        assert!(service
            .validate_file("../../../etc/passwd", "image/jpeg", 1024)
            .is_err());
    }

    #[tokio::test]
    async fn test_process_and_save_file() {
        use image::{ImageBuffer, Rgb};
        use std::io::Cursor;

        let temp_dir = TempDir::new().unwrap();
        let service = FileUploadService::new_profile_picture_service(
            temp_dir.path().to_string_lossy().to_string(),
        );

        // Create a valid 10x10 pixel RGB image
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_fn(10, 10, |_, _| Rgb([255, 0, 0]));

        // Encode to JPEG format
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Jpeg).unwrap();
        let test_image_data = buffer.into_inner();

        let result = service
            .process_and_save_file("test.jpg", "image/jpeg", test_image_data)
            .await;

        if let Err(ref e) = result {
            eprintln!("Error processing file: {:?}", e);
        }
        assert!(result.is_ok(), "File processing failed: {:?}", result.err());
        let result = result.unwrap();
        assert!(result.filename.ends_with(".jpg"));
        assert!(result.file_size > 0);
        assert_eq!(result.mime_type, "image/jpeg");
    }
}
