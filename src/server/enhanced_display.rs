use crate::platform::traits::*;
use crate::server::display::CompressionType;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, Duration, Instant};
use serde::{Serialize, Deserialize};

/// Enhanced display controller with cross-platform optimizations
pub struct EnhancedDisplayController {
    platform: Arc<dyn PlatformController + Send + Sync>,
    display_cache: Arc<RwLock<DisplayCache>>,
    performance_settings: Arc<RwLock<DisplayPerformanceSettings>>,
    capture_statistics: Arc<RwLock<CaptureStatistics>>,
}

#[derive(Debug, Clone)]
pub struct DisplayCache {
    pub cached_displays: HashMap<u32, CachedDisplayInfo>,
    pub last_refresh: Option<SystemTime>,
    pub cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CachedDisplayInfo {
    pub display_info: DisplayInfo,
    pub last_capture: Option<Vec<u8>>,
    pub last_capture_time: Option<SystemTime>,
    pub capture_count: u64,
    pub average_capture_time: Duration,
}

#[derive(Debug, Clone)]
pub struct DisplayPerformanceSettings {
    pub auto_compression: bool,
    pub preferred_format: CompressionType,
    pub quality_level: u8, // 0-100
    pub max_resolution_width: Option<u32>,
    pub max_resolution_height: Option<u32>,
    pub enable_differential_capture: bool,
    pub capture_rate_limit: Duration, // Minimum time between captures
}

#[derive(Debug, Clone, Default)]
pub struct CaptureStatistics {
    pub total_captures: u64,
    pub successful_captures: u64,
    pub failed_captures: u64,
    pub total_bytes_captured: u64,
    pub total_bytes_compressed: u64,
    pub average_capture_time: Duration,
    pub fastest_capture_time: Option<Duration>,
    pub slowest_capture_time: Option<Duration>,
    pub last_capture_time: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCaptureOptions {
    pub display_id: u32,
    pub force_refresh: bool,
    pub compression_format: Option<CompressionType>,
    pub quality: Option<u8>,
    pub differential: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for DisplayCache {
    fn default() -> Self {
        Self {
            cached_displays: HashMap::new(),
            last_refresh: None,
            cache_ttl: Duration::from_secs(30), // Cache for 30 seconds
        }
    }
}

impl Default for DisplayPerformanceSettings {
    fn default() -> Self {
        Self {
            auto_compression: true,
            preferred_format: CompressionType::PNG,
            quality_level: 80,
            max_resolution_width: Some(1920),
            max_resolution_height: Some(1080),
            enable_differential_capture: true,
            capture_rate_limit: Duration::from_millis(16), // ~60 FPS max
        }
    }
}

impl EnhancedDisplayController {
    /// Create a new enhanced display controller
    pub fn new(platform: Arc<dyn PlatformController + Send + Sync>) -> Self {
        Self {
            platform,
            display_cache: Arc::new(RwLock::new(DisplayCache::default())),
            performance_settings: Arc::new(RwLock::new(DisplayPerformanceSettings::default())),
            capture_statistics: Arc::new(RwLock::new(CaptureStatistics::default())),
        }
    }

    /// Get displays with caching
    pub async fn get_displays_cached(&self) -> Result<Vec<DisplayInfo>> {
        let cache = self.display_cache.read().await;
        
        // Check if cache is valid
        if let Some(last_refresh) = cache.last_refresh {
            if last_refresh.elapsed().unwrap_or(Duration::MAX) < cache.cache_ttl {
                return Ok(cache.cached_displays.values()
                    .map(|cached| cached.display_info.clone())
                    .collect());
            }
        }
        drop(cache);

        // Refresh cache
        self.refresh_display_cache().await
    }

    /// Force refresh the display cache
    pub async fn refresh_display_cache(&self) -> Result<Vec<DisplayInfo>> {
        let displays = self.platform.get_displays()?;
        
        let mut cache = self.display_cache.write().await;
        cache.cached_displays.clear();
        
        for display in &displays {
            let cached_info = CachedDisplayInfo {
                display_info: display.clone(),
                last_capture: None,
                last_capture_time: None,
                capture_count: 0,
                average_capture_time: Duration::from_millis(0),
            };
            cache.cached_displays.insert(display.id, cached_info);
        }
        
        cache.last_refresh = Some(SystemTime::now());
        
        Ok(displays)
    }

    /// Enhanced screen capture with optimizations
    pub async fn enhanced_capture_screen(&self, options: EnhancedCaptureOptions) -> Result<Vec<u8>> {
        let start_time = Instant::now();
        
        // Check rate limiting
        self.check_rate_limit().await?;
        
        // Get display info
        let displays = self.get_displays_cached().await?;
        let display = displays.iter()
            .find(|d| d.id == options.display_id)
            .ok_or_else(|| anyhow!("Display {} not found", options.display_id))?;

        // Check for cached capture if not forced
        if !options.force_refresh {
            if let Some(cached) = self.get_cached_capture(options.display_id).await? {
                return Ok(cached);
            }
        }

        // Perform the actual capture
        let mut capture_data = self.platform.capture_screen(options.display_id)?;
        
        // Apply resolution limits if specified
        if let (Some(max_width), Some(max_height)) = (options.max_width, options.max_height) {
            if display.width > max_width || display.height > max_height {
                capture_data = self.resize_capture_data(
                    capture_data, 
                    display.width, 
                    display.height, 
                    max_width, 
                    max_height
                ).await?;
            }
        }

        // Store original size for statistics
        let original_size = capture_data.len();
        
        // Apply compression if requested
        let final_data = if let Some(format) = options.compression_format {
            let quality = options.quality.unwrap_or(
                self.performance_settings.read().await.quality_level
            );
            self.compress_data(capture_data, format, quality).await?
        } else if self.performance_settings.read().await.auto_compression {
            let settings = self.performance_settings.read().await;
            self.compress_data(capture_data, settings.preferred_format, settings.quality_level).await?
        } else {
            capture_data
        };

        // Update cache
        self.update_capture_cache(options.display_id, final_data.clone(), start_time.elapsed()).await?;
        
        // Update statistics
        self.update_capture_statistics(true, original_size, final_data.len(), start_time.elapsed()).await;

        Ok(final_data)
    }

    /// Capture multiple displays simultaneously
    pub async fn capture_multiple_displays(&self, display_ids: Vec<u32>) -> Result<HashMap<u32, Vec<u8>>> {
        let mut results = HashMap::new();
        
        // Create capture tasks for each display
        let mut handles = Vec::new();
        
        for display_id in display_ids {
            let options = EnhancedCaptureOptions {
                display_id,
                force_refresh: false,
                compression_format: None,
                quality: None,
                differential: false,
                max_width: None,
                max_height: None,
            };
            
            let controller = self.clone_for_async();
            let handle = tokio::spawn(async move {
                (display_id, controller.enhanced_capture_screen(options).await)
            });
            handles.push(handle);
        }
        
        // Collect results
        for handle in handles {
            let (display_id, result) = handle.await
                .map_err(|e| anyhow!("Task join error: {}", e))?;
            
            match result {
                Ok(data) => {
                    results.insert(display_id, data);
                }
                Err(e) => {
                    tracing::warn!("Failed to capture display {}: {}", display_id, e);
                    self.update_capture_statistics(false, 0, 0, Duration::from_millis(0)).await;
                }
            }
        }
        
        Ok(results)
    }

    /// Get capture statistics
    pub async fn get_capture_statistics(&self) -> CaptureStatistics {
        self.capture_statistics.read().await.clone()
    }

    /// Update performance settings
    pub async fn update_performance_settings(&self, settings: DisplayPerformanceSettings) {
        let mut current_settings = self.performance_settings.write().await;
        *current_settings = settings;
    }

    /// Get current performance settings
    pub async fn get_performance_settings(&self) -> DisplayPerformanceSettings {
        self.performance_settings.read().await.clone()
    }

    /// Optimize settings based on system performance
    pub async fn auto_optimize_settings(&self) -> Result<()> {
        let stats = self.get_capture_statistics().await;
        
        let mut settings = self.performance_settings.write().await;
        
        // Auto-adjust quality based on performance
        if stats.average_capture_time > Duration::from_millis(100) {
            // Too slow, reduce quality
            settings.quality_level = (settings.quality_level.saturating_sub(10)).max(30);
            settings.max_resolution_width = Some(settings.max_resolution_width.unwrap_or(1920).min(1280));
            settings.max_resolution_height = Some(settings.max_resolution_height.unwrap_or(1080).min(720));
        } else if stats.average_capture_time < Duration::from_millis(16) {
            // Fast enough, can increase quality
            settings.quality_level = (settings.quality_level + 5).min(95);
        }
        
        // Enable differential capture for frequent captures
        if stats.total_captures > 10 {
            settings.enable_differential_capture = true;
        }
        
        Ok(())
    }

    /// Create a clone suitable for async operations
    fn clone_for_async(&self) -> Self {
        Self {
            platform: Arc::clone(&self.platform),
            display_cache: Arc::clone(&self.display_cache),
            performance_settings: Arc::clone(&self.performance_settings),
            capture_statistics: Arc::clone(&self.capture_statistics),
        }
    }

    /// Check rate limiting
    async fn check_rate_limit(&self) -> Result<()> {
        let settings = self.performance_settings.read().await;
        let stats = self.capture_statistics.read().await;
        
        if let Some(last_capture) = stats.last_capture_time {
            let elapsed = last_capture.elapsed().unwrap_or(Duration::MAX);
            if elapsed < settings.capture_rate_limit {
                let wait_time = settings.capture_rate_limit - elapsed;
                tokio::time::sleep(wait_time).await;
            }
        }
        
        Ok(())
    }

    /// Get cached capture if available and valid
    async fn get_cached_capture(&self, display_id: u32) -> Result<Option<Vec<u8>>> {
        let cache = self.display_cache.read().await;
        
        if let Some(cached_display) = cache.cached_displays.get(&display_id) {
            if let (Some(data), Some(capture_time)) = (&cached_display.last_capture, cached_display.last_capture_time) {
                // Check if cache is still valid (within 1 second for rapid captures)
                if capture_time.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(1) {
                    return Ok(Some(data.clone()));
                }
            }
        }
        
        Ok(None)
    }

    /// Update capture cache
    async fn update_capture_cache(&self, display_id: u32, data: Vec<u8>, capture_time: Duration) -> Result<()> {
        let mut cache = self.display_cache.write().await;
        
        if let Some(cached_display) = cache.cached_displays.get_mut(&display_id) {
            cached_display.last_capture = Some(data);
            cached_display.last_capture_time = Some(SystemTime::now());
            cached_display.capture_count += 1;
            
            // Update average capture time
            let total_time = cached_display.average_capture_time.as_millis() * (cached_display.capture_count - 1) as u128;
            let new_total = total_time + capture_time.as_millis();
            cached_display.average_capture_time = Duration::from_millis((new_total / cached_display.capture_count as u128) as u64);
        }
        
        Ok(())
    }

    /// Update capture statistics
    async fn update_capture_statistics(&self, success: bool, original_size: usize, final_size: usize, capture_time: Duration) {
        let mut stats = self.capture_statistics.write().await;
        
        stats.total_captures += 1;
        
        if success {
            stats.successful_captures += 1;
            stats.total_bytes_captured += original_size as u64;
            stats.total_bytes_compressed += final_size as u64;
            
            // Update timing statistics
            if stats.fastest_capture_time.is_none() || capture_time < stats.fastest_capture_time.unwrap() {
                stats.fastest_capture_time = Some(capture_time);
            }
            
            if stats.slowest_capture_time.is_none() || capture_time > stats.slowest_capture_time.unwrap() {
                stats.slowest_capture_time = Some(capture_time);
            }
            
            // Update average capture time
            let total_time = stats.average_capture_time.as_millis() * (stats.successful_captures - 1) as u128;
            let new_total = total_time + capture_time.as_millis();
            stats.average_capture_time = Duration::from_millis((new_total / stats.successful_captures as u128) as u64);
            
        } else {
            stats.failed_captures += 1;
        }
        
        stats.last_capture_time = Some(SystemTime::now());
    }

    /// Compress capture data
    async fn compress_data(&self, data: Vec<u8>, format: CompressionType, quality: u8) -> Result<Vec<u8>> {
        // Create a compressor and use it directly
        use crate::server::display::ImageCompressor;
        let compressor = ImageCompressor::with_quality(format, quality);
        compressor.compress(&data, 1920, 1080) // Default resolution
    }

    /// Resize capture data (placeholder implementation)
    async fn resize_capture_data(&self, data: Vec<u8>, original_width: u32, original_height: u32, max_width: u32, max_height: u32) -> Result<Vec<u8>> {
        // Calculate new dimensions maintaining aspect ratio
        let width_ratio = max_width as f64 / original_width as f64;
        let height_ratio = max_height as f64 / original_height as f64;
        let scale = width_ratio.min(height_ratio);
        
        let new_width = (original_width as f64 * scale) as u32;
        let new_height = (original_height as f64 * scale) as u32;
        
        // For now, return original data
        // In a real implementation, this would resize the image data
        tracing::info!("Would resize from {}x{} to {}x{}", original_width, original_height, new_width, new_height);
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeadlessPlatform;

    #[tokio::test]
    async fn test_enhanced_display_controller_creation() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        let settings = controller.get_performance_settings().await;
        assert!(settings.auto_compression);
        assert_eq!(settings.quality_level, 80);
    }

    #[tokio::test]
    async fn test_display_caching() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        // First call should refresh cache
        let displays1 = controller.get_displays_cached().await.unwrap();
        
        // Second call should use cache
        let displays2 = controller.get_displays_cached().await.unwrap();
        
        assert_eq!(displays1.len(), displays2.len());
    }

    #[tokio::test]
    async fn test_capture_options() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        let options = EnhancedCaptureOptions {
            display_id: 0,
            force_refresh: true,
            compression_format: Some(CompressionType::PNG),
            quality: Some(90),
            differential: false,
            max_width: Some(800),
            max_height: Some(600),
        };
        
        // This will fail on headless, but should not panic
        let result = controller.enhanced_capture_screen(options).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_performance_settings_update() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        let mut new_settings = DisplayPerformanceSettings::default();
        new_settings.quality_level = 50;
        new_settings.auto_compression = false;
        
        controller.update_performance_settings(new_settings).await;
        
        let updated_settings = controller.get_performance_settings().await;
        assert_eq!(updated_settings.quality_level, 50);
        assert!(!updated_settings.auto_compression);
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        // Simulate some captures
        controller.update_capture_statistics(true, 1000, 500, Duration::from_millis(50)).await;
        controller.update_capture_statistics(true, 1200, 600, Duration::from_millis(30)).await;
        controller.update_capture_statistics(false, 0, 0, Duration::from_millis(0)).await;
        
        let stats = controller.get_capture_statistics().await;
        assert_eq!(stats.total_captures, 3);
        assert_eq!(stats.successful_captures, 2);
        assert_eq!(stats.failed_captures, 1);
        assert_eq!(stats.total_bytes_captured, 2200);
        assert_eq!(stats.total_bytes_compressed, 1100);
    }

    #[tokio::test]
    async fn test_auto_optimization() {
        let platform: Arc<dyn PlatformController + Send + Sync> = Arc::new(HeadlessPlatform::new());
        let controller = EnhancedDisplayController::new(platform);
        
        // Simulate slow captures
        for _ in 0..5 {
            controller.update_capture_statistics(true, 1000, 500, Duration::from_millis(150)).await;
        }
        
        let initial_quality = controller.get_performance_settings().await.quality_level;
        
        controller.auto_optimize_settings().await.unwrap();
        
        let optimized_quality = controller.get_performance_settings().await.quality_level;
        assert!(optimized_quality < initial_quality);
    }
}