use std::sync::Arc;
use reqwest::header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceValidation {
    pub is_valid: bool,
    pub normalized_url: String,
    pub provider_id: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFormat {
    pub format_id: String,
    pub resolution: String,
    pub extension: String,
    pub bitrate: Option<u64>,
    pub fps: Option<u32>,
    pub estimated_size: Option<u64>,
    pub supports_resume: bool,
    pub audio_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub source_url: String,
    pub provider_id: String,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<u64>,
    pub author: Option<String>,
    pub formats: Vec<MediaFormat>,
}

#[async_trait::async_trait]
pub trait MediaProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    async fn validate_source(&self, source_url: &str) -> Result<SourceValidation, AppError>;

    async fn resolve_metadata(&self, source_url: &str) -> Result<MediaMetadata, AppError>;

    async fn resolve_formats(&self, source_url: &str) -> Result<Vec<MediaFormat>, AppError>;
}

pub struct DirectHttpProvider {
    client: reqwest::Client,
}

impl DirectHttpProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("BulkVideoDownloader/1.0.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

#[async_trait::async_trait]
impl MediaProvider for DirectHttpProvider {
    fn provider_id(&self) -> &'static str {
        "direct_http"
    }

    async fn validate_source(&self, source_url: &str) -> Result<SourceValidation, AppError> {
        let trimmed = source_url.trim();
        if trimmed.is_empty() {
            return Ok(SourceValidation {
                is_valid: false,
                normalized_url: String::new(),
                provider_id: self.provider_id().to_string(),
                error_message: Some("URL is empty".into()),
            });
        }

        // Check for accidental concatenation or corrupted scheme patterns
        let lower = trimmed.to_lowercase();
        if lower.contains("http//") || lower.contains("https//") {
            return Ok(SourceValidation {
                is_valid: false,
                normalized_url: trimmed.to_string(),
                provider_id: self.provider_id().to_string(),
                error_message: Some("URL contains corrupted scheme syntax (e.g. 'http//' or 'https//' without colon).".into()),
            });
        }

        let http_count = lower.match_indices("http://").count() + lower.match_indices("https://").count();
        if http_count > 1 {
            return Ok(SourceValidation {
                is_valid: false,
                normalized_url: trimmed.to_string(),
                provider_id: self.provider_id().to_string(),
                error_message: Some("URL contains multiple embedded web addresses. Please separate them onto new lines.".into()),
            });
        }

        match Url::parse(trimmed) {
            Ok(parsed) => {
                let scheme = parsed.scheme();
                if scheme != "http" && scheme != "https" {
                    return Ok(SourceValidation {
                        is_valid: false,
                        normalized_url: trimmed.to_string(),
                        provider_id: self.provider_id().to_string(),
                        error_message: Some(format!("Unsupported protocol: '{}'. Only HTTP/HTTPS is permitted.", scheme)),
                    });
                }

                // Check path for double slashes or embedded domain indicators
                let path_str = parsed.path();
                if path_str.starts_with("//") {
                    return Ok(SourceValidation {
                        is_valid: false,
                        normalized_url: trimmed.to_string(),
                        provider_id: self.provider_id().to_string(),
                        error_message: Some("URL contains invalid malformed double slashes in path.".into()),
                    });
                }

                // Validate host
                let host_str = match parsed.host_str() {
                    Some(h) if !h.trim().is_empty() => h.trim(),
                    _ => {
                        return Ok(SourceValidation {
                            is_valid: false,
                            normalized_url: trimmed.to_string(),
                            provider_id: self.provider_id().to_string(),
                            error_message: Some("URL is missing a valid domain or host name.".into()),
                        });
                    }
                };

                // Host must not contain embedded slashes or protocol fragments
                if host_str.contains('/') || host_str.contains('\\') || host_str.contains(':') || host_str.ends_with("http") || host_str.ends_with("https") {
                    return Ok(SourceValidation {
                        is_valid: false,
                        normalized_url: trimmed.to_string(),
                        provider_id: self.provider_id().to_string(),
                        error_message: Some("Host contains invalid characters, slashes or embedded protocol fragments.".into()),
                    });
                }

                // Check for localhost or IP address vs domain structure
                let is_localhost = host_str.eq_ignore_ascii_case("localhost");
                let is_ip = parsed.host().map_or(false, |h| matches!(h, url::Host::Ipv4(_) | url::Host::Ipv6(_)));

                if !is_localhost && !is_ip {
                    // Standard internet domain: must have at least one dot separating name and TLD
                    if !host_str.contains('.') {
                        return Ok(SourceValidation {
                            is_valid: false,
                            normalized_url: trimmed.to_string(),
                            provider_id: self.provider_id().to_string(),
                            error_message: Some(format!("Invalid domain '{}': missing top-level domain extension.", host_str)),
                        });
                    }

                    // TLD (after last dot) must be at least 2 ASCII alphabetic characters
                    let tld = host_str.rsplit('.').next().unwrap_or("");
                    if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
                        return Ok(SourceValidation {
                            is_valid: false,
                            normalized_url: trimmed.to_string(),
                            provider_id: self.provider_id().to_string(),
                            error_message: Some(format!("Invalid domain '{}': invalid top-level domain '.{}'.", host_str, tld)),
                        });
                    }

                    // Domain cannot start or end with '.' or '-'
                    if host_str.starts_with('.') || host_str.ends_with('.') || host_str.starts_with('-') || host_str.ends_with('-') {
                        return Ok(SourceValidation {
                            is_valid: false,
                            normalized_url: trimmed.to_string(),
                            provider_id: self.provider_id().to_string(),
                            error_message: Some(format!("Invalid domain format '{}'.", host_str)),
                        });
                    }
                }

                Ok(SourceValidation {
                    is_valid: true,
                    normalized_url: parsed.to_string(),
                    provider_id: self.provider_id().to_string(),
                    error_message: None,
                })
            }
            Err(e) => Ok(SourceValidation {
                is_valid: false,
                normalized_url: trimmed.to_string(),
                provider_id: self.provider_id().to_string(),
                error_message: Some(format!("Invalid URL syntax: {}", e)),
            }),
        }
    }

    async fn resolve_metadata(&self, source_url: &str) -> Result<MediaMetadata, AppError> {
        let parsed = Url::parse(source_url).map_err(|e| AppError::ValidationError {
            code: "INVALID_URL".into(),
            message: format!("Cannot parse URL: {}", e),
            action: "Check URL formatting.".into(),
        })?;

        // Extract title from URL path or fallback
        let path_segments: Vec<&str> = parsed.path_segments().map(|c| c.collect()).unwrap_or_default();
        let raw_filename = path_segments.last().copied().unwrap_or("media_download");
        let fallback_title = raw_filename.split('.').next().unwrap_or("video_download").to_string();

        let mut title = fallback_title;
        let mut estimated_size: Option<u64> = None;
        let mut extension = "mp4".to_string();
        let mut supports_resume = true;

        if let Ok(resp) = self.client.head(source_url).send().await {
            if resp.status().is_success() {
                if let Some(cl) = resp.headers().get(CONTENT_LENGTH) {
                    if let Ok(cl_str) = cl.to_str() {
                        estimated_size = cl_str.parse::<u64>().ok();
                    }
                }
                if let Some(ct) = resp.headers().get(CONTENT_TYPE) {
                    if let Ok(ct_str) = ct.to_str() {
                        if ct_str.contains("webm") {
                            extension = "webm".into();
                        } else if ct_str.contains("quicktime") || ct_str.contains("mov") {
                            extension = "mov".into();
                        } else if ct_str.contains("x-matroska") || ct_str.contains("mkv") {
                            extension = "mkv".into();
                        } else if ct_str.contains("audio") || ct_str.contains("mpeg") {
                            extension = "mp3".into();
                        }
                    }
                }
                if let Some(cd) = resp.headers().get(CONTENT_DISPOSITION) {
                    if let Ok(cd_str) = cd.to_str() {
                        if let Some(idx) = cd_str.find("filename=") {
                            let part = &cd_str[idx + 9..];
                            let cleaned = part.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                            if !cleaned.is_empty() {
                                title = cleaned.split('.').next().unwrap_or(cleaned).to_string();
                            }
                        }
                    }
                }
                if let Some(ar) = resp.headers().get(ACCEPT_RANGES) {
                    if let Ok(ar_str) = ar.to_str() {
                        supports_resume = ar_str.eq_ignore_ascii_case("bytes");
                    }
                }
            }
        }

        let formats = vec![
            MediaFormat {
                format_id: "original".into(),
                resolution: "Original / Source".into(),
                extension: extension.clone(),
                bitrate: None,
                fps: None,
                estimated_size,
                supports_resume,
                audio_only: extension == "mp3",
            },
            MediaFormat {
                format_id: "1080p".into(),
                resolution: "1080p".into(),
                extension: "mp4".into(),
                bitrate: Some(4500),
                fps: Some(30),
                estimated_size,
                supports_resume,
                audio_only: false,
            },
            MediaFormat {
                format_id: "720p".into(),
                resolution: "720p".into(),
                extension: "mp4".into(),
                bitrate: Some(2500),
                fps: Some(30),
                estimated_size,
                supports_resume,
                audio_only: false,
            },
        ];

        Ok(MediaMetadata {
            source_url: source_url.to_string(),
            provider_id: self.provider_id().to_string(),
            title,
            description: None,
            thumbnail_url: None,
            duration_seconds: None,
            author: None,
            formats,
        })
    }

    async fn resolve_formats(&self, source_url: &str) -> Result<Vec<MediaFormat>, AppError> {
        let meta = self.resolve_metadata(source_url).await?;
        Ok(meta.formats)
    }
}

pub mod ytdlp;
pub use ytdlp::{YtDlpProvider, YtDlpStatus};

pub struct ProviderRegistry {
    direct: Arc<dyn MediaProvider>,
    ytdlp: Arc<dyn MediaProvider>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            direct: Arc::new(DirectHttpProvider::new()),
            ytdlp: Arc::new(ytdlp::YtDlpProvider),
        }
    }

    pub fn get_provider_for_url(&self, url: &str) -> Arc<dyn MediaProvider> {
        if ytdlp::YtDlpProvider::is_supported_social_url(url) {
            self.ytdlp.clone()
        } else {
            self.direct.clone()
        }
    }
}
