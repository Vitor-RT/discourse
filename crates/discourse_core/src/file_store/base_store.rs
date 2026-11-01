use rutie::{AnyObject, Class, Object, RString, VM, Integer};

pub const TEMPORARY_UPLOAD_PREFIX: &str = "temp/";
pub const CACHE_MAXIMUM_SIZE: usize = 500;
pub const CACHE_EVICT_COUNT: usize = 100;

pub fn get_depth_for(id: i64) -> usize {
    if id > 0 {
        let depth = (id as f64 / 1_000.0).log(16.0).ceil() as usize;
        if depth > 0 {
            return depth;
        }
    }
    0
}

pub fn get_path_for(file_type: &str, id: i64, sha: &str, extension: &str) -> String {
    let depth = get_depth_for(id);
    let mut tree = String::new();
    if depth > 0 {
        for c in sha.chars().take(depth) {
            tree.push(c);
            tree.push('/');
        }
    }
    format!("{}/{}X/{}{}{}", file_type, depth + 1, tree, sha, extension)
}

rutie::class!(RubyBaseStore);

rutie::methods!(
    RubyBaseStore,
    _itself,

    fn ruby_get_depth_for(id: Integer) -> Integer {
        let result = get_depth_for(id.unwrap().to_i64());
        Integer::new(result as i64)
    }

    fn ruby_get_path_for(file_type: RString, id: Integer, sha: RString, extension: RString) -> RString {
        let result = get_path_for(
            &file_type.unwrap().to_string(),
            id.unwrap().to_i64(),
            &sha.unwrap().to_string(),
            &extension.unwrap().to_string()
        );
        RString::new_utf8(&result)
    }
);

#[no_mangle]
pub extern "C" fn Init_base_store() {
    Class::new("FileStore::BaseStore", None).define(|klass| {
        klass.def_self("get_depth_for", ruby_get_depth_for);
        klass.def_self("get_path_for", ruby_get_path_for);
    });
}
