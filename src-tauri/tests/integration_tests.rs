use bulk_video_downloader_lib::domain::{
    render_filename, sanitize_filename, Batch, BatchStatus, CompletionPolicy, DownloadStatus,
    DownloadTask,
};
use bulk_video_downloader_lib::persistence::Database;
use bulk_video_downloader_lib::providers::{DirectHttpProvider, MediaProvider};
use bulk_video_downloader_lib::recovery::RecoveryManager;

#[tokio::test]
async fn test_database_lifecycle_and_integrity() {
    let db = Database::new_in_memory().expect("Failed to create in-memory database");
    assert!(db.check_integrity().expect("Integrity check failed"));

    let batch = Batch {
        id: "test-batch-1".into(),
        name: "Test Batch".into(),
        status: BatchStatus::Ready,
        batch_size: 10,
        concurrency: 3,
        completion_policy: CompletionPolicy::TerminalCompletion,
        failure_policy: "PAUSE_BATCH".into(),
        profile_id: None,
        destination_directory: "./downloads".into(),
        naming_template: "{title}_{quality}.{format}".into(),
        created_at: "2026-09-16T00:00:00Z".into(),
        started_at: None,
        completed_at: None,
        updated_at: "2026-09-16T00:00:00Z".into(),
    };

    db.insert_batch(&batch).expect("Insert batch failed");
    let fetched = db.get_batch("test-batch-1").expect("Get batch failed");
    assert!(fetched.is_some());
    let fetched_batch = fetched.unwrap();
    assert_eq!(fetched_batch.name, "Test Batch");
    assert_eq!(fetched_batch.batch_size, 10);
    assert_eq!(fetched_batch.concurrency, 3);

    let task = DownloadTask {
        id: "task-1".into(),
        batch_id: "test-batch-1".into(),
        source_url: "https://example.com/video1.mp4".into(),
        provider_id: Some("direct_http".into()),
        source_id: None,
        title: Some("video1".into()),
        thumbnail_url: None,
        status: DownloadStatus::Queued,
        priority: 0,
        queue_position: 1,
        selected_format_id: Some("original".into()),
        selected_quality: Some("1080p".into()),
        selected_extension: Some("mp4".into()),
        output_directory: "./downloads".into(),
        output_filename: Some("video1_1080p.mp4".into()),
        output_path: None,
        partial_path: None,
        bytes_downloaded: 0,
        total_bytes: Some(1024000),
        progress: 0.0,
        speed_bytes_per_second: None,
        eta_seconds: None,
        retry_count: 0,
        max_retries: 3,
        next_retry_at: None,
        last_error_code: None,
        last_error_message: None,
        created_at: "2026-09-16T00:00:00Z".into(),
        started_at: None,
        completed_at: None,
        updated_at: "2026-09-16T00:00:00Z".into(),
    };

    db.insert_downloads(&[task]).expect("Insert download task failed");
    let fetched_task = db.get_download("task-1").expect("Get task failed");
    assert!(fetched_task.is_some());
    assert_eq!(fetched_task.unwrap().source_url, "https://example.com/video1.mp4");

    // Test duplicate detection in DB
    assert!(db.check_existing_url("https://example.com/video1.mp4").unwrap());
    assert!(!db.check_existing_url("https://example.com/nonexistent.mp4").unwrap());
}

#[tokio::test]
async fn test_direct_http_provider_validation() {
    let provider = DirectHttpProvider::new();

    // Valid HTTPS URL
    let val_valid = provider.validate_source("https://example.com/movie.mp4").await.unwrap();
    assert!(val_valid.is_valid);
    assert_eq!(val_valid.normalized_url, "https://example.com/movie.mp4");

    // Invalid scheme (e.g. ftp, file)
    let val_ftp = provider.validate_source("ftp://example.com/movie.mp4").await.unwrap();
    assert!(!val_ftp.is_valid);

    // Malformed and corrupted URLs
    let val_corrupted = provider
        .validate_source("https://www.facehttps//www.facebook.com/share/v/14q37Qd4QN7/book.com/share/v/14q37Qd4QN7/")
        .await
        .unwrap();
    assert!(!val_corrupted.is_valid, "Corrupted facehttps URL must be rejected");

    let val_nodot = provider.validate_source("https://nodotdomain/video.mp4").await.unwrap();
    assert!(!val_nodot.is_valid, "No-dot domain must be rejected");

    let val_nested = provider
        .validate_source("https://site.com/vid.mp4https://other.com/vid.mp4")
        .await
        .unwrap();
    assert!(!val_nested.is_valid, "Nested concatenated URLs must be rejected");

    // Genuine Facebook video URL
    let val_fb = provider
        .validate_source("https://www.facebook.com/share/v/14q37Qd4QN7/")
        .await
        .unwrap();
    assert!(val_fb.is_valid, "Valid Facebook URL must pass validation");

    // Empty URL
    let val_empty = provider.validate_source("   ").await.unwrap();
    assert!(!val_empty.is_valid);
}

#[test]
fn test_naming_and_sanitization() {
    let raw = "Crazy? Video: <Title> | 2026*";
    let sanitized = sanitize_filename(raw);
    assert_eq!(sanitized, "Crazy_ Video_ _Title_ _ 2026_");

    let rendered = render_filename(
        "{batch_id}_{index}_{title}.{format}",
        "My Video",
        "vid123",
        "DirectHTTP",
        "1080p",
        "mp4",
        "B01",
        1,
    );
    assert_eq!(rendered, "B01_1_My Video.mp4");
}

#[test]
fn test_recovery_reconciliation_on_clean_state() {
    let db = std::sync::Arc::new(Database::new_in_memory().unwrap());
    let recovery = RecoveryManager::new(db);
    let summary = recovery.run_startup_recovery().expect("Recovery run failed");
    assert!(summary.integrity_ok);
    assert_eq!(summary.tasks_reconciled, 0);
}

#[tokio::test]
async fn test_social_media_provider_routing_and_identification() {
    use bulk_video_downloader_lib::providers::{ProviderRegistry, YtDlpProvider};

    // 1. URL pattern recognition
    let yt_url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    let tt_url = "https://www.tiktok.com/@user/video/7123456789";
    let ig_url = "https://www.instagram.com/reel/C8abc123/";
    let x_url = "https://x.com/rustlang/status/123456789";
    let fb_url = "https://www.facebook.com/share/v/14q37Qd4QN7/";
    let corrupted_fb_url = "https://www.facehttps//www.facebook.com/share/v/14q37Qd4QN7/book.com/share/v/14q37Qd4QN7/";
    let direct_url = "https://example.com/assets/media.mp4";

    assert!(YtDlpProvider::is_supported_social_url(yt_url));
    assert!(YtDlpProvider::is_supported_social_url(tt_url));
    assert!(YtDlpProvider::is_supported_social_url(ig_url));
    assert!(YtDlpProvider::is_supported_social_url(x_url));
    assert!(YtDlpProvider::is_supported_social_url(fb_url));
    assert!(!YtDlpProvider::is_supported_social_url(corrupted_fb_url), "Corrupted URL must not match social provider");
    assert!(!YtDlpProvider::is_supported_social_url(direct_url));

    // 2. Platform identification
    assert_eq!(YtDlpProvider::identify_platform(yt_url), "YouTube");
    assert_eq!(YtDlpProvider::identify_platform(tt_url), "TikTok (No Watermark)");
    assert_eq!(YtDlpProvider::identify_platform(ig_url), "Instagram");
    assert_eq!(YtDlpProvider::identify_platform(x_url), "X / Twitter");
    assert_eq!(YtDlpProvider::identify_platform(fb_url), "Facebook");

    // 3. ProviderRegistry routing
    let registry = ProviderRegistry::new();
    let yt_provider = registry.get_provider_for_url(yt_url);
    let direct_provider = registry.get_provider_for_url(direct_url);

    assert_eq!(yt_provider.provider_id(), "yt_dlp");
    assert_eq!(direct_provider.provider_id(), "direct_http");

    // 4. Status check execution without panic
    let status = YtDlpProvider::check_status();
    assert!(!status.message.is_empty());
}

#[tokio::test]
async fn test_embedded_ffmpeg_and_ytdlp_availability() {
    use bulk_video_downloader_lib::media::MediaProcessor;
    use bulk_video_downloader_lib::providers::YtDlpProvider;

    // Test embedded FFmpeg detection
    let ffmpeg_status = MediaProcessor::check_ffmpeg();
    println!("FFmpeg status: {:?}", ffmpeg_status);
    assert!(ffmpeg_status.is_available, "FFmpeg binary should be detected in embedded bin folder");
    assert!(ffmpeg_status.binary_path.is_some(), "Binary path must be resolved");

    // Test embedded yt-dlp detection
    let ytdlp_status = YtDlpProvider::check_status();
    println!("yt-dlp status: {:?}", ytdlp_status);
    assert!(ytdlp_status.is_available, "yt-dlp binary should be detected in embedded bin folder");
    assert!(ytdlp_status.binary_path.is_some(), "Binary path must be resolved");
}
