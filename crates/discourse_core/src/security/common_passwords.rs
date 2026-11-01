use magnus::{define_class, define_module, function, method, prelude::*, Error, Value, RClass};

fn common_password(class: RClass, password: Option<String>) -> Result<bool, Error> {
    match password {
        Some(pwd) => {
            if pwd.trim().is_empty() {
                return Ok(false);
            }
<<<<<<< HEAD
            
=======

>>>>>>> origin/rust-migration-7847495834899956704
            let password_list: Value = class.funcall("password_list", ())?;
            let result: bool = password_list.funcall("include?", (pwd,))?;
            Ok(result)
        }
        None => Ok(false)
    }
}

pub fn init() -> Result<(), Error> {
    let class = magnus::class::object().const_get::<_, RClass>("CommonPasswords")?;
<<<<<<< HEAD
    
    // class.undef_default_alloc_func();
    
    class.singleton_class()?.define_method("common_password?", method!(common_password, 1))?;
    
=======

    // class.undef_default_alloc_func();

    class.singleton_class()?.define_method("common_password?", method!(common_password, 1))?;

>>>>>>> origin/rust-migration-7847495834899956704
    Ok(())
}
