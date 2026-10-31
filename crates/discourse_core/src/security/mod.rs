use magnus::Error;

pub mod admin_confirmation;
pub mod admin_constraint;
pub mod common_passwords;
pub mod csrf_token_verifier;

pub fn init() -> Result<(), Error> {
    // wait, we need to load classes first before opening them, or define them if they don't exist. 
    // Usually rails autoloads, but if we do this in an initializer, the classes might not exist yet.
    // If they don't exist, const_get fails.
    
    // Instead of doing it at require time, we could have a `DiscourseCore.init` method in rust, 
    // and call it after the classes are loaded.
    
    Ok(())
}
