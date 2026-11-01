use rutie::{AnyObject, Class, Object, RString, Boolean, VM, NilClass};

pub const CANNED_ACL_PUBLIC_READ: &str = "public-read";
pub const CANNED_ACL_PRIVATE: &str = "private";
pub const MAX_CONTENT_DISPOSITION_BYTES: usize = 1600;

pub struct S3Store;

impl S3Store {
    pub fn acl_option_value(secure: bool, s3_use_acls: bool) -> Option<&'static str> {
        if !s3_use_acls {
            return None;
        }
        if secure {
            Some(CANNED_ACL_PRIVATE)
        } else {
            Some(CANNED_ACL_PUBLIC_READ)
        }
    }
}

rutie::class!(RubyS3Store);

rutie::methods!(
    RubyS3Store,
    _itself,

    fn ruby_acl_option_value(secure: Boolean, s3_use_acls: Boolean) -> AnyObject {
        match S3Store::acl_option_value(secure.unwrap().to_bool(), s3_use_acls.unwrap().to_bool()) {
            Some(val) => RString::new_utf8(val).into(),
            None => NilClass::new().into()
        }
    }
);

#[no_mangle]
pub extern "C" fn Init_s3_store() {
    Class::new("FileStore::S3Store", None).define(|klass| {
        klass.def_self("acl_option_value", ruby_acl_option_value);
    });
}
