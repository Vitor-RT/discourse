use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq)]
pub enum Browser {
    Edge,
    Opera,
    SamsungBrowser,
    UcBrowser,
    QqBrowser,
    BaiduBrowser,
    KaiosBrowser,
    Ie,
    Firefox,
    Chrome,
    AndroidBrowser,
    Safari,
    Discoursehub,
    Unknown,
}

impl Browser {
    pub fn parse(user_agent: &str) -> Self {
        static EDGE_RE: OnceLock<Regex> = OnceLock::new();
        static OPERA_RE: OnceLock<Regex> = OnceLock::new();
        static SAMSUNG_RE: OnceLock<Regex> = OnceLock::new();
        static UC_RE: OnceLock<Regex> = OnceLock::new();
        static QQ_RE: OnceLock<Regex> = OnceLock::new();
        static BAIDU_RE: OnceLock<Regex> = OnceLock::new();
        static KAIOS_RE: OnceLock<Regex> = OnceLock::new();
        static IE_RE: OnceLock<Regex> = OnceLock::new();
        static FIREFOX_RE: OnceLock<Regex> = OnceLock::new();
        static CHROME_RE: OnceLock<Regex> = OnceLock::new();
        static ANDROID_RE: OnceLock<Regex> = OnceLock::new();
        static SAFARI_RE: OnceLock<Regex> = OnceLock::new();
        static DISCOURSE_RE: OnceLock<Regex> = OnceLock::new();

<<<<<<< HEAD
        if EDGE_RE
            .get_or_init(|| Regex::new(r"(?i)Edg").unwrap())
            .is_match(user_agent)
        {
            Browser::Edge
        } else if OPERA_RE
            .get_or_init(|| Regex::new(r"(?i)Opera|OPR").unwrap())
            .is_match(user_agent)
        {
            Browser::Opera
        } else if SAMSUNG_RE
            .get_or_init(|| Regex::new(r"(?i)SamsungBrowser/").unwrap())
            .is_match(user_agent)
        {
            Browser::SamsungBrowser
        } else if UC_RE
            .get_or_init(|| Regex::new(r"(?i)(?:UCBrowser|UC Browser)[/ ]").unwrap())
            .is_match(user_agent)
        {
            Browser::UcBrowser
        } else if QQ_RE
            .get_or_init(|| Regex::new(r"(?i)(?:MQQBrowser|QQBrowser)/").unwrap())
            .is_match(user_agent)
        {
            Browser::QqBrowser
        } else if BAIDU_RE
            .get_or_init(|| Regex::new(r"(?i)(?:BIDUBrowser|BaiduBrowser)/").unwrap())
            .is_match(user_agent)
        {
            Browser::BaiduBrowser
        } else if KAIOS_RE
            .get_or_init(|| Regex::new(r"(?i)KaiOS/").unwrap())
            .is_match(user_agent)
        {
            Browser::KaiosBrowser
        } else if IE_RE
            .get_or_init(|| Regex::new(r"(?i)MSIE|Trident|IEMobile").unwrap())
            .is_match(user_agent)
        {
            Browser::Ie
        } else if FIREFOX_RE
            .get_or_init(|| Regex::new(r"(?i)Firefox|FxiOS").unwrap())
            .is_match(user_agent)
        {
            Browser::Firefox
        } else if CHROME_RE
            .get_or_init(|| Regex::new(r"(?i)Chrome|CriOS").unwrap())
            .is_match(user_agent)
        {
            Browser::Chrome
        } else if ANDROID_RE
            .get_or_init(|| Regex::new(r"(?i)Android.+Version/[\d.]+.+Safari").unwrap())
            .is_match(user_agent)
        {
            Browser::AndroidBrowser
        } else if SAFARI_RE
            .get_or_init(|| Regex::new(r"(?i)Safari").unwrap())
            .is_match(user_agent)
        {
            Browser::Safari
        } else if DISCOURSE_RE
            .get_or_init(|| Regex::new(r"(?i)Discourse").unwrap())
            .is_match(user_agent)
        {
=======
        if EDGE_RE.get_or_init(|| Regex::new(r"(?i)Edg").unwrap()).is_match(user_agent) {
            Browser::Edge
        } else if OPERA_RE.get_or_init(|| Regex::new(r"(?i)Opera|OPR").unwrap()).is_match(user_agent) {
            Browser::Opera
        } else if SAMSUNG_RE.get_or_init(|| Regex::new(r"(?i)SamsungBrowser/").unwrap()).is_match(user_agent) {
            Browser::SamsungBrowser
        } else if UC_RE.get_or_init(|| Regex::new(r"(?i)(?:UCBrowser|UC Browser)[/ ]").unwrap()).is_match(user_agent) {
            Browser::UcBrowser
        } else if QQ_RE.get_or_init(|| Regex::new(r"(?i)(?:MQQBrowser|QQBrowser)/").unwrap()).is_match(user_agent) {
            Browser::QqBrowser
        } else if BAIDU_RE.get_or_init(|| Regex::new(r"(?i)(?:BIDUBrowser|BaiduBrowser)/").unwrap()).is_match(user_agent) {
            Browser::BaiduBrowser
        } else if KAIOS_RE.get_or_init(|| Regex::new(r"(?i)KaiOS/").unwrap()).is_match(user_agent) {
            Browser::KaiosBrowser
        } else if IE_RE.get_or_init(|| Regex::new(r"(?i)MSIE|Trident|IEMobile").unwrap()).is_match(user_agent) {
            Browser::Ie
        } else if FIREFOX_RE.get_or_init(|| Regex::new(r"(?i)Firefox|FxiOS").unwrap()).is_match(user_agent) {
            Browser::Firefox
        } else if CHROME_RE.get_or_init(|| Regex::new(r"(?i)Chrome|CriOS").unwrap()).is_match(user_agent) {
            Browser::Chrome
        } else if ANDROID_RE.get_or_init(|| Regex::new(r"(?i)Android.+Version/[\d.]+.+Safari").unwrap()).is_match(user_agent) {
            Browser::AndroidBrowser
        } else if SAFARI_RE.get_or_init(|| Regex::new(r"(?i)Safari").unwrap()).is_match(user_agent) {
            Browser::Safari
        } else if DISCOURSE_RE.get_or_init(|| Regex::new(r"(?i)Discourse").unwrap()).is_match(user_agent) {
>>>>>>> origin/rust-migration-7847495834899956704
            Browser::Discoursehub
        } else {
            Browser::Unknown
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Device {
    Android,
    Chromebook,
    Ipad,
    Iphone,
    Ipod,
    Mobile,
    Mac,
    Linux,
    Windows,
    Unknown,
}

impl Device {
    pub fn parse(user_agent: &str) -> Self {
        static ANDROID_RE: OnceLock<Regex> = OnceLock::new();
        static CHROMEBOOK_RE: OnceLock<Regex> = OnceLock::new();
        static IPAD_RE: OnceLock<Regex> = OnceLock::new();
        static IPHONE_RE: OnceLock<Regex> = OnceLock::new();
        static IPOD_RE: OnceLock<Regex> = OnceLock::new();
        static MOBILE_RE: OnceLock<Regex> = OnceLock::new();
        static MAC_RE: OnceLock<Regex> = OnceLock::new();
        static LINUX_RE: OnceLock<Regex> = OnceLock::new();
        static WINDOWS_RE: OnceLock<Regex> = OnceLock::new();

<<<<<<< HEAD
        if ANDROID_RE
            .get_or_init(|| Regex::new(r"(?i)Android").unwrap())
            .is_match(user_agent)
        {
            Device::Android
        } else if CHROMEBOOK_RE
            .get_or_init(|| Regex::new(r"(?i)CrOS").unwrap())
            .is_match(user_agent)
        {
            Device::Chromebook
        } else if IPAD_RE
            .get_or_init(|| Regex::new(r"(?i)iPad").unwrap())
            .is_match(user_agent)
        {
            Device::Ipad
        } else if IPHONE_RE
            .get_or_init(|| Regex::new(r"(?i)iPhone").unwrap())
            .is_match(user_agent)
        {
            Device::Iphone
        } else if IPOD_RE
            .get_or_init(|| Regex::new(r"(?i)iPod").unwrap())
            .is_match(user_agent)
        {
            Device::Ipod
        } else if MOBILE_RE
            .get_or_init(|| Regex::new(r"(?i)Mobile").unwrap())
            .is_match(user_agent)
        {
            Device::Mobile
        } else if MAC_RE
            .get_or_init(|| Regex::new(r"(?i)Macintosh").unwrap())
            .is_match(user_agent)
        {
            Device::Mac
        } else if LINUX_RE
            .get_or_init(|| Regex::new(r"(?i)Linux").unwrap())
            .is_match(user_agent)
        {
            Device::Linux
        } else if WINDOWS_RE
            .get_or_init(|| Regex::new(r"(?i)Windows").unwrap())
            .is_match(user_agent)
        {
=======
        if ANDROID_RE.get_or_init(|| Regex::new(r"(?i)Android").unwrap()).is_match(user_agent) {
            Device::Android
        } else if CHROMEBOOK_RE.get_or_init(|| Regex::new(r"(?i)CrOS").unwrap()).is_match(user_agent) {
            Device::Chromebook
        } else if IPAD_RE.get_or_init(|| Regex::new(r"(?i)iPad").unwrap()).is_match(user_agent) {
            Device::Ipad
        } else if IPHONE_RE.get_or_init(|| Regex::new(r"(?i)iPhone").unwrap()).is_match(user_agent) {
            Device::Iphone
        } else if IPOD_RE.get_or_init(|| Regex::new(r"(?i)iPod").unwrap()).is_match(user_agent) {
            Device::Ipod
        } else if MOBILE_RE.get_or_init(|| Regex::new(r"(?i)Mobile").unwrap()).is_match(user_agent) {
            Device::Mobile
        } else if MAC_RE.get_or_init(|| Regex::new(r"(?i)Macintosh").unwrap()).is_match(user_agent) {
            Device::Mac
        } else if LINUX_RE.get_or_init(|| Regex::new(r"(?i)Linux").unwrap()).is_match(user_agent) {
            Device::Linux
        } else if WINDOWS_RE.get_or_init(|| Regex::new(r"(?i)Windows").unwrap()).is_match(user_agent) {
>>>>>>> origin/rust-migration-7847495834899956704
            Device::Windows
        } else {
            Device::Unknown
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Os {
    Android,
    Chromeos,
    Ios,
    Macos,
    Linux,
    Windows,
    Unknown,
}

impl Os {
    pub fn parse(user_agent: &str) -> Self {
        static ANDROID_RE: OnceLock<Regex> = OnceLock::new();
        static CHROMEOS_RE: OnceLock<Regex> = OnceLock::new();
        static IOS_RE: OnceLock<Regex> = OnceLock::new();
        static MACOS_RE: OnceLock<Regex> = OnceLock::new();
        static LINUX_RE: OnceLock<Regex> = OnceLock::new();
        static WINDOWS_RE: OnceLock<Regex> = OnceLock::new();

<<<<<<< HEAD
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
=======
        if ANDROID_RE.get_or_init(|| Regex::new(r"(?i)Android").unwrap()).is_match(user_agent) {
            Os::Android
        } else if CHROMEOS_RE.get_or_init(|| Regex::new(r"(?i)CrOS").unwrap()).is_match(user_agent) {
            Os::Chromeos
        } else if IOS_RE.get_or_init(|| Regex::new(r"(?i)iPhone|iPad|iPod|Darwin").unwrap()).is_match(user_agent) {
            Os::Ios
        } else if MACOS_RE.get_or_init(|| Regex::new(r"(?i)Macintosh").unwrap()).is_match(user_agent) {
            Os::Macos
        } else if LINUX_RE.get_or_init(|| Regex::new(r"(?i)Linux").unwrap()).is_match(user_agent) {
            Os::Linux
        } else if WINDOWS_RE.get_or_init(|| Regex::new(r"(?i)Windows").unwrap()).is_match(user_agent) {
>>>>>>> origin/rust-migration-7847495834899956704
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
<<<<<<< HEAD
        assert_eq!(
            Browser::parse(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0"
            ),
            Browser::Firefox
        );
=======
        assert_eq!(Browser::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0"), Browser::Firefox);
>>>>>>> origin/rust-migration-7847495834899956704
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
