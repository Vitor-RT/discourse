use crate::csp::Directives;

#[derive(Debug, Clone, Default)]
pub struct CspSettings {
    pub force_https: bool,
    pub report_only: bool,
    pub report_uri: Option<String>,
    pub use_s3: bool,
    pub s3_cdn_url: Option<String>,
    pub s3_asset_cdn_url: Option<String>,
    pub cdn_url: Option<String>,
    pub base_path: String,
    pub frame_ancestors: bool,
    pub embed_any_origin: bool,
    pub embeddable_hosts: Vec<String>,
}

pub struct DefaultPolicy {
    pub directives: Directives,
}

impl DefaultPolicy {
    pub fn new(settings: &CspSettings) -> Self {
        let mut directives = Directives::new();

        if settings.force_https && !settings.report_only {
            directives.insert("upgrade-insecure-requests".to_string(), vec![]);
        }

        directives.insert("base-uri".to_string(), vec!["'self'".to_string()]);
        directives.insert("object-src".to_string(), vec!["'none'".to_string()]);
        directives.insert("script-src".to_string(), Self::script_src(settings));
        directives.insert("worker-src".to_string(), Self::worker_src(settings));

        if Self::restrict_embed(settings) {
            directives.insert(
                "frame-ancestors".to_string(),
                Self::frame_ancestors(settings),
            );
        }

        directives.insert("manifest-src".to_string(), vec!["'self'".to_string()]);

        if let Some(uri) = &settings.report_uri {
            if !uri.is_empty() {
                directives.insert("report-uri".to_string(), vec![uri.clone()]);
            }
        }

        Self { directives }
    }

    fn script_src(settings: &CspSettings) -> Vec<String> {
        let mut sources = vec![
            "'strict-dynamic'".to_string(),
            "'wasm-unsafe-eval'".to_string(),
        ];
        if let Some(uri) = &settings.report_uri {
            if !uri.is_empty() {
                sources.push("'report-sample'".to_string());
            }
        }
        sources
    }

    fn worker_src(settings: &CspSettings) -> Vec<String> {
        let mut sources = vec!["'self'".to_string(), "blob:".to_string()];
        sources.extend(Self::worker_asset_host(settings));
        sources
    }

    fn worker_asset_host(settings: &CspSettings) -> Vec<String> {
        if settings.use_s3 && settings.s3_cdn_url.is_some() {
            let s3_cdn = if let Some(asset_url) = &settings.s3_asset_cdn_url {
                if !asset_url.is_empty() {
                    asset_url.clone()
                } else {
                    settings.s3_cdn_url.clone().unwrap()
                }
            } else {
                settings.s3_cdn_url.clone().unwrap()
            };
            vec![format!("{}/assets/", s3_cdn)]
        } else if let Some(cdn_url) = &settings.cdn_url {
            if !cdn_url.is_empty() {
                vec![format!("{}{}/assets/", cdn_url, settings.base_path)]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn frame_ancestors(settings: &CspSettings) -> Vec<String> {
        let mut ancestors = vec!["'self'".to_string()];
        for host in &settings.embeddable_hosts {
            ancestors.push(format!("https://{}", host));
        }
        ancestors
    }

    fn restrict_embed(settings: &CspSettings) -> bool {
        settings.frame_ancestors && !settings.embed_any_origin
    }
}
