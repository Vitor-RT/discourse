use once_cell::sync::OnceCell;
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Browser {
    Edge,
    Opera,
    Chrome,
    Safari,
    Firefox,
    IE,
    UCBrowser,
    Samsung,
    Bots,
    Unknown,
}

static EDGE_RE: OnceCell<Regex> = OnceCell::new();
static OPERA_RE: OnceCell<Regex> = OnceCell::new();
static CHROME_RE: OnceCell<Regex> = OnceCell::new();
static SAFARI_RE: OnceCell<Regex> = OnceCell::new();
static FIREFOX_RE: OnceCell<Regex> = OnceCell::new();
static IE_RE: OnceCell<Regex> = OnceCell::new();
static UCBROWSER_RE: OnceCell<Regex> = OnceCell::new();
static SAMSUNG_RE: OnceCell<Regex> = OnceCell::new();

impl Browser {
    pub fn parse(user_agent: &str) -> Self {
        if EDGE_RE
            .get_or_init(|| Regex::new(r"(?i)Edg|Edge|EdgA|EdgiOS").unwrap())
            .is_match(user_agent)
        {
            Browser::Edge
        } else if OPERA_RE
            .get_or_init(|| Regex::new(r"(?i)OPR|Opera").unwrap())
            .is_match(user_agent)
        {
            Browser::Opera
        } else if SAMSUNG_RE
            .get_or_init(|| Regex::new(r"(?i)SamsungBrowser").unwrap())
            .is_match(user_agent)
        {
            Browser::Samsung
        } else if UCBROWSER_RE
            .get_or_init(|| Regex::new(r"(?i)UCBrowser|UCWEB").unwrap())
            .is_match(user_agent)
        {
            Browser::UCBrowser
        } else if CHROME_RE
            .get_or_init(|| Regex::new(r"(?i)Chrome|CriOS").unwrap())
            .is_match(user_agent)
        {
            Browser::Chrome
        } else if FIREFOX_RE
            .get_or_init(|| Regex::new(r"(?i)Firefox|FxiOS").unwrap())
            .is_match(user_agent)
        {
            Browser::Firefox
        } else if SAFARI_RE
            .get_or_init(|| Regex::new(r"(?i)Safari").unwrap())
            .is_match(user_agent)
        {
            Browser::Safari
        } else if IE_RE
            .get_or_init(|| Regex::new(r"(?i)MSIE|Trident").unwrap())
            .is_match(user_agent)
        {
            Browser::IE
        } else {
            Browser::Unknown
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Device {
    iPhone,
    iPad,
    iPod,
    Android,
    WindowsPhone,
    Mac,
    Windows,
    Linux,
    Unknown,
}

static IPHONE_RE: OnceCell<Regex> = OnceCell::new();
static IPAD_RE: OnceCell<Regex> = OnceCell::new();
static IPOD_RE: OnceCell<Regex> = OnceCell::new();
static ANDROID_DEV_RE: OnceCell<Regex> = OnceCell::new();
static WINPHONE_RE: OnceCell<Regex> = OnceCell::new();
static MAC_RE: OnceCell<Regex> = OnceCell::new();
static WIN_RE: OnceCell<Regex> = OnceCell::new();
static LINUX_DEV_RE: OnceCell<Regex> = OnceCell::new();

impl Device {
    pub fn parse(user_agent: &str) -> Self {
        if IPHONE_RE
            .get_or_init(|| Regex::new(r"(?i)iPhone").unwrap())
            .is_match(user_agent)
        {
            Device::iPhone
        } else if IPAD_RE
            .get_or_init(|| Regex::new(r"(?i)iPad").unwrap())
            .is_match(user_agent)
        {
            Device::iPad
        } else if IPOD_RE
            .get_or_init(|| Regex::new(r"(?i)iPod").unwrap())
            .is_match(user_agent)
        {
            Device::iPod
        } else if WINPHONE_RE
            .get_or_init(|| Regex::new(r"(?i)Windows Phone").unwrap())
            .is_match(user_agent)
        {
            Device::WindowsPhone
        } else if ANDROID_DEV_RE
            .get_or_init(|| Regex::new(r"(?i)Android").unwrap())
            .is_match(user_agent)
        {
            Device::Android
        } else if MAC_RE
            .get_or_init(|| Regex::new(r"(?i)Macintosh").unwrap())
            .is_match(user_agent)
        {
            Device::Mac
        } else if WIN_RE
            .get_or_init(|| Regex::new(r"(?i)Windows").unwrap())
            .is_match(user_agent)
        {
            Device::Windows
        } else if LINUX_DEV_RE
            .get_or_init(|| Regex::new(r"(?i)Linux").unwrap())
            .is_match(user_agent)
        {
            Device::Linux
        } else {
            Device::Unknown
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Os {
    Android,
    Chromeos,
    Ios,
    Macos,
    Linux,
    Windows,
    Unknown,
}

static ANDROID_RE: OnceCell<Regex> = OnceCell::new();
static CHROMEOS_RE: OnceCell<Regex> = OnceCell::new();
static IOS_RE: OnceCell<Regex> = OnceCell::new();
static MACOS_RE: OnceCell<Regex> = OnceCell::new();
static LINUX_RE: OnceCell<Regex> = OnceCell::new();
static WINDOWS_RE: OnceCell<Regex> = OnceCell::new();

impl Os {
    pub fn parse(user_agent: &str) -> Self {
        if ANDROID_RE
            .get_or_init(|| Regex::new(r"(?i)Android").unwrap())
            .is_match(user_agent)
        {
            Os::Android
        } else if CHROMEOS_RE
            .get_or_init(|| Regex::new(r"(?i)CrOS").unwrap())
            .is_match(user_agent)
        {
            Os::Chromeos
        } else if IOS_RE
            .get_or_init(|| Regex::new(r"(?i)iPhone|iPad|iPod|Darwin").unwrap())
            .is_match(user_agent)
        {
            Os::Ios
        } else if MACOS_RE
            .get_or_init(|| Regex::new(r"(?i)Macintosh").unwrap())
            .is_match(user_agent)
        {
            Os::Macos
        } else if LINUX_RE
            .get_or_init(|| Regex::new(r"(?i)Linux").unwrap())
            .is_match(user_agent)
        {
            Os::Linux
        } else if WINDOWS_RE
            .get_or_init(|| Regex::new(r"(?i)Windows").unwrap())
            .is_match(user_agent)
        {
            Os::Windows
        } else {
            Os::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_parse() {
        assert_eq!(Browser::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36 Edg/100.0.1185.39"), Browser::Edge);
        assert_eq!(Browser::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36 OPR/86.0.4363.59"), Browser::Opera);
        assert_eq!(Browser::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36"), Browser::Chrome);
        assert_eq!(Browser::parse("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Safari/605.1.15"), Browser::Safari);
        assert_eq!(
            Browser::parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0"
            ),
            Browser::Firefox
        );
        assert_eq!(Browser::parse("Unknown User Agent"), Browser::Unknown);
    }

    #[test]
    fn test_device_parse() {
        assert_eq!(Device::parse("Mozilla/5.0 (Linux; Android 12; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.79 Mobile Safari/537.36"), Device::Android);
        assert_eq!(Device::parse("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36"), Device::Mac);
        assert_eq!(Device::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36"), Device::Windows);
        assert_eq!(Device::parse("Unknown User Agent"), Device::Unknown);
    }

    #[test]
    fn test_os_parse() {
        assert_eq!(Os::parse("Mozilla/5.0 (Linux; Android 12; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.79 Mobile Safari/537.36"), Os::Android);
        assert_eq!(Os::parse("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36"), Os::Macos);
        assert_eq!(Os::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36"), Os::Windows);
        assert_eq!(Os::parse("Unknown User Agent"), Os::Unknown);
    }
}
