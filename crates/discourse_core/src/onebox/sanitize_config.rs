use ammonia::Builder;

pub struct SanitizeConfig;

impl SanitizeConfig {
    pub fn onebox() -> Builder<'static> {
        let mut builder = Builder::default();
        builder.add_tags(vec![
            "audio", "details", "embed", "iframe", "source", "video", "svg", "path", "use",
        ]);
        builder.add_tag_attributes("a", vec!["target"]);
        builder.add_tag_attributes("audio", vec!["controls", "controlslist"]);
        builder.add_tag_attributes("embed", vec!["height", "src", "type", "width"]);
        builder.add_tag_attributes(
            "iframe",
            vec![
                "allowfullscreen",
                "frameborder",
                "height",
                "scrolling",
                "src",
                "width",
                "data-original-href",
                "data-unsanitized-src",
            ],
        );
        builder.add_tag_attributes("source", vec!["src", "type"]);
        builder.add_tag_attributes(
            "video",
            vec![
                "controls",
                "height",
                "loop",
                "width",
                "autoplay",
                "muted",
                "poster",
                "controlslist",
                "playsinline",
            ],
        );
        builder.add_tag_attributes("path", vec!["d", "fill-rule"]);
        builder.add_tag_attributes("svg", vec!["aria-hidden", "width", "height", "viewbox"]);
        builder.add_tag_attributes("use", vec!["href"]);

        // Note: For ammonia to support wildcards like data-*, we'd use generic attributes or a custom Node Filter, ammonia doesn't do data-* by default easily without `add_generic_attributes`, which exists.
        // It's a simplification.
        builder
    }
}
