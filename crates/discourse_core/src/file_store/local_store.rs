use rutie::{AnyObject, Class, Object, RString, Boolean, VM};

pub struct LocalStore;

impl LocalStore {
    pub fn external() -> bool {
        false
    }

    pub fn is_relative(url: &str, relative_base_url: &str) -> bool {
        !url.is_empty() && url.starts_with(relative_base_url)
    }

    pub fn tombstone_dir(public_dir: &str, relative_base_url: &str) -> String {
        format!("{}{}", public_dir, relative_base_url.replace("/uploads/", "/uploads/tombstone/"))
    }
}

rutie::class!(RubyLocalStore);

rutie::methods!(
    RubyLocalStore,
    _itself,

    fn ruby_external() -> Boolean {
        Boolean::new(LocalStore::external())
    }

    fn ruby_is_relative(url: RString, relative_base_url: RString) -> Boolean {
        let result = LocalStore::is_relative(&url.unwrap().to_string(), &relative_base_url.unwrap().to_string());
        Boolean::new(result)
    }

    fn ruby_tombstone_dir(public_dir: RString, relative_base_url: RString) -> RString {
        let result = LocalStore::tombstone_dir(&public_dir.unwrap().to_string(), &relative_base_url.unwrap().to_string());
        RString::new_utf8(&result)
    }
);

#[no_mangle]
pub extern "C" fn Init_local_store() {
    Class::new("FileStore::LocalStore", None).define(|klass| {
        klass.def_self("external?", ruby_external);
        klass.def_self("is_relative?", ruby_is_relative);
        klass.def_self("tombstone_dir", ruby_tombstone_dir);
    });
}
