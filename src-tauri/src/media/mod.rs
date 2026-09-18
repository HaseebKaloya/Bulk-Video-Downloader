use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFmpegStatus {
    pub is_available: bool,
    pub ffmpeg_version: Option<String>,
    pub ffprobe_available: bool,
    pub binary_path: Option<String>,
    pub message: String,
}

pub struct MediaProcessor;

impl MediaProcessor {
    pub fn find_binary(name: &str) -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            if name.ends_with(".exe") {
                name.to_string()
            } else {
                format!("{}.exe", name)
            }
        } else {
            name.to_string()
        };

        // 1. Check next to running executable and Tauri resource folders (Windows NSIS/portable, Linux AppImage/deb, macOS)
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                let candidates = [
                    exe_dir.join(&exe_name),
                    exe_dir.join("bin").join(&exe_name),
                    exe_dir.join("resources").join("bin").join(&exe_name),
                    exe_dir.join("resources").join(&exe_name),
                    exe_dir.join("..").join("Resources").join("bin").join(&exe_name),
                    exe_dir.join("..").join("Resources").join(&exe_name),
                    exe_dir.join("..").join("lib").join(&exe_name),
                    exe_dir.join("..").join("lib").join("bin").join(&exe_name),
                    exe_dir.join("..").join("bin").join(&exe_name),
                    exe_dir.join("..").join("..").join("bin").join(&exe_name),
                ];
                for candidate in &candidates {
                    if candidate.is_file() {
                        return candidate.clone();
                    }
                }
            }
        }

        // 2. Check current working directory and project root folders (Dev & CI mode)
        if let Ok(cwd) = std::env::current_dir() {
            let candidates = [
                cwd.join(&exe_name),
                cwd.join("bin").join(&exe_name),
                cwd.join("src-tauri").join("bin").join(&exe_name),
                cwd.join("..").join("bin").join(&exe_name),
                cwd.join("..").join("src-tauri").join("bin").join(&exe_name),
                cwd.join("..").join(&exe_name),
                cwd.join("ffmpeg").join("bin").join(&exe_name),
                cwd.join("..").join("ffmpeg").join("bin").join(&exe_name),
            ];
            for candidate in &candidates {
                if candidate.is_file() {
                    return candidate.clone();
                }
            }
        }

        // 3. Fallback to system PATH
        PathBuf::from(name)
    }

    pub fn create_command<P: AsRef<Path>>(bin: P) -> Command {
        let mut cmd = Command::new(bin.as_ref());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd
    }

    pub fn check_ffmpeg() -> FFmpegStatus {
        let ffmpeg_bin = Self::find_binary("ffmpeg");
        let ffprobe_bin = Self::find_binary("ffprobe");

        let ffmpeg_out = Self::create_command(&ffmpeg_bin).arg("-version").output();
        let ffprobe_out = Self::create_command(&ffprobe_bin).arg("-version").output();

        let ffprobe_available = ffprobe_out.map(|o| o.status.success()).unwrap_or(false);

        match ffmpeg_out {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let first_line = stdout.lines().next().unwrap_or("ffmpeg installed").to_string();
                let path_str = ffmpeg_bin.to_string_lossy().to_string();

                FFmpegStatus {
                    is_available: true,
                    ffmpeg_version: Some(first_line),
                    ffprobe_available,
                    binary_path: Some(path_str),
                    message: "FFmpeg is active and available for media post-processing.".to_string(),
                }
            }
            _ => FFmpegStatus {
                is_available: false,
                ffmpeg_version: None,
                ffprobe_available: false,
                binary_path: None,
                message: "FFmpeg was not detected. Place 'ffmpeg.exe' and 'ffprobe.exe' in a 'bin/' folder or install via 'winget install Gyan.FFmpeg.Essentials'.".to_string(),
            },
        }
    }

    pub fn extract_thumbnail(
        input_file: &Path,
        output_thumbnail: &Path,
        time_offset_secs: u64,
    ) -> Result<PathBuf, AppError> {
        let status = Self::check_ffmpeg();
        if !status.is_available {
            return Err(AppError::MediaProcessingError {
                code: "FFMPEG_NOT_FOUND".into(),
                message: "FFmpeg is required to generate video thumbnails.".into(),
                action: "Install FFmpeg or place ffmpeg.exe in the application bin folder.".into(),
            });
        }

        let ffmpeg_bin = Self::find_binary("ffmpeg");
        let time_arg = format!("00:00:{:02}", time_offset_secs.min(59));

        let out = Self::create_command(ffmpeg_bin)
            .arg("-y")
            .arg("-ss")
            .arg(&time_arg)
            .arg("-i")
            .arg(input_file)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg(output_thumbnail)
            .output()
            .map_err(|e| AppError::MediaProcessingError {
                code: "FFMPEG_EXEC_FAILED".into(),
                message: format!("Failed to execute FFmpeg thumbnail extraction: {}", e),
                action: "Check FFmpeg installation.".into(),
            })?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AppError::MediaProcessingError {
                code: "FFMPEG_THUMBNAIL_FAILED".into(),
                message: format!("FFmpeg thumbnail extraction failed: {}", stderr),
                action: "Verify the downloaded video file is valid.".into(),
            });
        }

        Ok(output_thumbnail.to_path_buf())
    }

    pub fn convert_container(
        input_file: &Path,
        output_file: &Path,
    ) -> Result<PathBuf, AppError> {
        let status = Self::check_ffmpeg();
        if !status.is_available {
            return Err(AppError::MediaProcessingError {
                code: "FFMPEG_NOT_FOUND".into(),
                message: "FFmpeg is required to convert media containers.".into(),
                action: "Install FFmpeg or place ffmpeg.exe in the application bin folder.".into(),
            });
        }

        let ffmpeg_bin = Self::find_binary("ffmpeg");

        let out = Self::create_command(ffmpeg_bin)
            .arg("-y")
            .arg("-i")
            .arg(input_file)
            .arg("-c")
            .arg("copy")
            .arg(output_file)
            .output()
            .map_err(|e| AppError::MediaProcessingError {
                code: "FFMPEG_EXEC_FAILED".into(),
                message: format!("Failed to execute FFmpeg remux: {}", e),
                action: "Check FFmpeg installation.".into(),
            })?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AppError::MediaProcessingError {
                code: "FFMPEG_CONVERT_FAILED".into(),
                message: format!("FFmpeg container conversion failed: {}", stderr),
                action: "Verify the source file codecs are compatible.".into(),
            });
        }

        Ok(output_file.to_path_buf())
    }
}
