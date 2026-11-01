use super::validations;
use magnus::{function, prelude::*, Error, Ruby};

fn split_s3_bucket(s3_bucket: String) -> (String, Option<String>) {
    validations::split_s3_bucket(&s3_bucket)
}

fn validate_cors_origins(new_val: String) -> Result<(), magnus::Error> {
    validations::validate_cors_origins(&new_val)
        .map_err(|e| magnus::Error::new(Ruby::get().unwrap().exception_standard_error(), e))
}

fn validate_slow_down_crawler_user_agents(new_val: String) -> Result<(), magnus::Error> {
    validations::validate_slow_down_crawler_user_agents(&new_val)
        .map_err(|e| magnus::Error::new(Ruby::get().unwrap().exception_standard_error(), e))
}

fn validate_bucket_setting(
    setting_name: String,
    upload_bucket: String,
    backup_bucket: String,
) -> Result<(), magnus::Error> {
    validations::validate_bucket_setting(&setting_name, &upload_bucket, &backup_bucket)
        .map_err(|e| magnus::Error::new(Ruby::get().unwrap().exception_standard_error(), e))
}

#[magnus::init]
pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("DiscourseCore")?;
    let site_settings = module.define_module("SiteSettings")?;
    let validations = site_settings.define_module("Validations")?;
    validations
        .define_module_function("validate_cors_origins", function!(validate_cors_origins, 1))?;
    validations.define_module_function(
        "validate_slow_down_crawler_user_agents",
        function!(validate_slow_down_crawler_user_agents, 1),
    )?;
    validations.define_module_function("split_s3_bucket", function!(split_s3_bucket, 1))?;
    validations.define_module_function(
        "validate_bucket_setting",
        function!(validate_bucket_setting, 3),
    )?;

    Ok(())
}
