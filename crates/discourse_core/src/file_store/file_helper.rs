use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    static ref SUPPORTED_GRAVATAR_IMAGES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("jpg");
        set.insert("jpeg");
        set.insert("png");
        set.insert("gif");
        set
    };
    static ref SUPPORTED_IMAGES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("jpg");
        set.insert("jpeg");
        set.insert("png");
        set.insert("gif");
        set.insert("svg");
        set.insert("ico");
        set.insert("webp");
        set.insert("avif");
        set.insert("heic");
        set.insert("heif");
        set.insert("jxl");
        set
    };
    static ref INLINE_IMAGES: HashSet<&'static str> = {
        let mut set = SUPPORTED_IMAGES.clone();
        set.remove("svg");
        set
    };
    static ref SUPPORTED_AUDIO: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("mp3");
        set.insert("ogg");
        set.insert("oga");
        set.insert("opus");
        set.insert("wav");
        set.insert("m4a");
        set.insert("m4b");
        set.insert("m4p");
        set.insert("m4r");
        set.insert("aac");
        set.insert("flac");
        set
    };
    static ref SUPPORTED_VIDEO: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("mov");
        set.insert("mp4");
        set.insert("m4v");
        set.insert("mkv");
        set.insert("webm");
        set.insert("ogv");
        set.insert("avi");
        set
    };
}

pub struct FileHelper;

impl FileHelper {
    pub fn is_supported_image(filename: &str) -> bool {
        if let Some(ext) = filename.split('.').last() {
            SUPPORTED_IMAGES.contains(ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    pub fn is_supported_video(filename: &str) -> bool {
        if let Some(ext) = filename.split('.').last() {
            SUPPORTED_VIDEO.contains(ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    pub fn is_supported_audio(filename: &str) -> bool {
        if let Some(ext) = filename.split('.').last() {
            SUPPORTED_AUDIO.contains(ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    pub fn is_inline_image(filename: &str) -> bool {
        if let Some(ext) = filename.split('.').last() {
            INLINE_IMAGES.contains(ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    pub fn is_svg(filename: &str) -> bool {
        filename.to_lowercase().ends_with(".svg")
    }

    pub fn is_supported_media(filename: &str) -> bool {
        Self::is_supported_image(filename)
            || Self::is_supported_video(filename)
            || Self::is_supported_audio(filename)
    }

    pub fn sanitize_filename(filename: &str) -> String {
        let re = Regex::new(r#"[^\w\.\-]"#).unwrap();
        re.replace_all(filename, "_").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_helper_image_checks() {
        assert!(FileHelper::is_supported_image("test.png"));
        assert!(FileHelper::is_supported_image("photo.JPEG"));
        assert!(!FileHelper::is_supported_image("file.txt"));

        assert!(FileHelper::is_supported_video("video.mp4"));
        assert!(FileHelper::is_supported_audio("music.mp3"));

        assert_eq!(
            FileHelper::sanitize_filename("my file?.png"),
            "my_file_.png"
        );
    }
}
