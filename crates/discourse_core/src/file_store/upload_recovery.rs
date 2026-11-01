use rutie::{AnyObject, Class, Object, RString, Boolean, VM, Array};

pub struct UploadRecovery {
    dry_run: bool,
    stop_on_error: bool,
}

impl UploadRecovery {
    pub fn new(dry_run: bool, stop_on_error: bool) -> Self {
        Self {
            dry_run,
            stop_on_error,
        }
    }

    pub fn valid_sha1(sha1: &str) -> bool {
        !sha1.is_empty() && sha1.len() == 40
    }
}

rutie::class!(RubyUploadRecovery);

rutie::methods!(
    RubyUploadRecovery,
    _itself,

    fn ruby_valid_sha1(sha1: RString) -> Boolean {
        let result = UploadRecovery::valid_sha1(&sha1.unwrap().to_string());
        Boolean::new(result)
    }
);

#[no_mangle]
pub extern "C" fn Init_upload_recovery() {
    Class::new("UploadRecovery", None).define(|klass| {
        klass.def_self("valid_sha1?", ruby_valid_sha1);
    });
}
