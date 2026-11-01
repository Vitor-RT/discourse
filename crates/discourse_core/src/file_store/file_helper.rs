use lazy_static::lazy_static;
use regex::Regex;
use rutie::{AnyObject, Class, Object, RString, Boolean, VM, Array};
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
        set.insert("webm");
        set.insert("ogv");
        set.insert("m4v");
        set.insert("3gp");
        set.insert("avi");
        set.insert("mpeg");
        set
    };

    static ref INLINE_SAFE_FILES: HashSet<&'static str> = {
        let mut set = INLINE_IMAGES.clone();
        set.insert("pdf");
        for ext in SUPPORTED_VIDEO.iter() {
            set.insert(ext);
        }
        for ext in SUPPORTED_AUDIO.iter() {
            set.insert(ext);
        }
        set
    };

    static ref SUPPORTED_VIDEO_REGEXP: Regex = {
        let pattern = format!(r"(?i)\.({})\z", SUPPORTED_VIDEO.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };

    static ref SUPPORTED_AUDIO_REGEXP: Regex = {
        let pattern = format!(r"(?i)\.({})\z", SUPPORTED_AUDIO.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };

    static ref SUPPORTED_IMAGES_REGEXP: Regex = {
        let pattern = format!(r"(?i)\.({})\z", SUPPORTED_IMAGES.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };

    static ref INLINE_IMAGES_REGEXP: Regex = {
        let pattern = format!(r"(?i)\.({})\z", INLINE_IMAGES.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };

    static ref SUPPORTED_MEDIA_REGEXP: Regex = {
        let mut media = SUPPORTED_IMAGES.clone();
        for ext in SUPPORTED_AUDIO.iter() { media.insert(ext); }
        for ext in SUPPORTED_VIDEO.iter() { media.insert(ext); }
        let pattern = format!(r"(?i)\.({})\z", media.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };

    static ref SUPPORTED_PLAYABLE_MEDIA_REGEXP: Regex = {
        let mut media = SUPPORTED_AUDIO.clone();
        for ext in SUPPORTED_VIDEO.iter() { media.insert(ext); }
        let pattern = format!(r"(?i)\.({})\z", media.iter().copied().collect::<Vec<_>>().join("|"));
        Regex::new(&pattern).unwrap()
    };
}

pub struct FileHelper;

impl FileHelper {
    pub fn is_supported_image(filename: &str) -> bool {
        SUPPORTED_IMAGES_REGEXP.is_match(filename)
    }

    pub fn is_supported_video(filename: &str) -> bool {
        SUPPORTED_VIDEO_REGEXP.is_match(filename)
    }

    pub fn is_supported_audio(filename: &str) -> bool {
        SUPPORTED_AUDIO_REGEXP.is_match(filename)
    }

    pub fn is_inline_image(filename: &str) -> bool {
        INLINE_IMAGES_REGEXP.is_match(filename)
    }

    pub fn is_svg(filename: &str) -> bool {
        let svg_regex = Regex::new(r"(?i)\.svg\z").unwrap();
        svg_regex.is_match(filename)
    }

    pub fn is_supported_media(filename: &str) -> bool {
        SUPPORTED_MEDIA_REGEXP.is_match(filename)
    }

    pub fn is_supported_playable_media(filename: &str) -> bool {
        SUPPORTED_PLAYABLE_MEDIA_REGEXP.is_match(filename)
    }

    pub fn sanitize_filename(filename: &str) -> String {
        let mut name = filename.trim().to_string();

        if let Some(pos) = name.rfind(|c| c == '\\' || c == '/') {
            name = name[pos + 1..].to_string();
        }

        let re1 = Regex::new(r"[^\w\.\-]").unwrap();
        name = re1.replace_all(&name, "_").to_string();

        let re2 = Regex::new(r"_+").unwrap();
        name = re2.replace_all(&name, "_").to_string();

        name
    }

    pub fn is_inline_safe(filename: &str) -> bool {
        if filename.trim().is_empty() {
            return false;
        }
        let lower = filename.to_lowercase();
        INLINE_SAFE_FILES.iter().any(|ext| lower.ends_with(&format!(".{}", ext)))
    }
}

rutie::class!(RubyFileHelper);

rutie::methods!(
    RubyFileHelper,
    _itself,

    fn ruby_is_supported_image(filename: RString) -> Boolean {
        let result = FileHelper::is_supported_image(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_supported_video(filename: RString) -> Boolean {
        let result = FileHelper::is_supported_video(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_supported_audio(filename: RString) -> Boolean {
        let result = FileHelper::is_supported_audio(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_inline_image(filename: RString) -> Boolean {
        let result = FileHelper::is_inline_image(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_svg(filename: RString) -> Boolean {
        let result = FileHelper::is_svg(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_supported_media(filename: RString) -> Boolean {
        let result = FileHelper::is_supported_media(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_is_supported_playable_media(filename: RString) -> Boolean {
        let result = FileHelper::is_supported_playable_media(&filename.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_sanitize_filename(filename: RString) -> RString {
        let result = FileHelper::sanitize_filename(&filename.unwrap().to_string());
        RString::new_utf8(&result)
    }

    fn ruby_is_inline_safe(filename: RString) -> Boolean {
        let result = FileHelper::is_inline_safe(&filename.unwrap().to_string());
        Boolean::new(result)
    }
);

#[no_mangle]
pub extern "C" fn Init_file_helper() {
    Class::new("FileHelper", None).define(|klass| {
        klass.def_self("is_supported_image?", ruby_is_supported_image);
        klass.def_self("is_supported_video?", ruby_is_supported_video);
        klass.def_self("is_supported_audio?", ruby_is_supported_audio);
        klass.def_self("is_inline_image?", ruby_is_inline_image);
        klass.def_self("is_svg?", ruby_is_svg);
        klass.def_self("is_supported_media?", ruby_is_supported_media);
        klass.def_self("is_supported_playable_media?", ruby_is_supported_playable_media);
        klass.def_self("sanitize_filename", ruby_sanitize_filename);
        klass.def_self("is_inline_safe?", ruby_is_inline_safe);
    });
}
