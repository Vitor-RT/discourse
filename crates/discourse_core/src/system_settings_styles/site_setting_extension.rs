pub struct SiteSettingExtension {
    default_locale: String,
}

impl SiteSettingExtension {
    pub fn new() -> Self {
        Self {
            default_locale: String::from("en"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_setting_extension_initialization() {
        let extension = SiteSettingExtension::new();
        assert_eq!(extension.default_locale, "en");
    }
}
