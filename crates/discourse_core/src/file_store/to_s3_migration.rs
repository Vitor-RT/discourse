use rutie::{AnyObject, Class, Object, Boolean, VM, RString};

pub const UPLOAD_CONCURRENCY: usize = 4;
pub const MISSING_UPLOADS_RAKE_TASK_NAME: &str = "posts:missing_uploads";

pub struct ToS3Migration {
    dry_run: bool,
    current_db: String,
}

impl ToS3Migration {
    pub fn new(dry_run: bool, current_db: String) -> Self {
        Self {
            dry_run,
            current_db,
        }
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn get_current_db(&self) -> &str {
        &self.current_db
    }
}

rutie::class!(RubyToS3Migration);

rutie::methods!(
    RubyToS3Migration,
    _itself,

    fn ruby_is_dry_run(dry_run: Boolean, current_db: RString) -> Boolean {
        let migration = ToS3Migration::new(dry_run.unwrap().to_bool(), current_db.unwrap().to_string());
        Boolean::new(migration.is_dry_run())
    }
);

#[no_mangle]
pub extern "C" fn Init_to_s3_migration() {
    Class::new("FileStore::ToS3Migration", None).define(|klass| {
        klass.def_self("is_dry_run?", ruby_is_dry_run);
    });
}
