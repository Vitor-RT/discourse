use magnus::{method, prelude::*, TryConvert, Error, Value, RClass, RModule, RHash};

fn initialize_admin_constraint(rb_self: Value, args: &[Value]) -> Result<Value, Error> {
    let options = args.get(0);
    
    let require_master = if let Some(opt) = options {
        if opt.is_kind_of(magnus::class::hash()) {
            let hash = RHash::from_value(*opt).unwrap();
            let sym = magnus::Symbol::new("require_master");
            if let Some(val) = hash.get(sym) {
                if val.is_nil() { false } else { bool::try_convert(val).unwrap_or(true) }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    
    rb_self.funcall::<_, _, Value>("instance_variable_set", ("@require_master", require_master))?;
    Ok(rb_self)
}

fn matches(rb_self: Value, request: Value) -> Result<bool, Error> {
    let require_master_val: Value = rb_self.funcall("instance_variable_get", ("@require_master",))?;
    let require_master = if require_master_val.is_nil() { false } else { bool::try_convert(require_master_val).unwrap_or(true) };

    if require_master {
        let conn_mgmt = magnus::class::object().const_get::<_, RModule>("RailsMultisite")?
            .const_get::<_, RClass>("ConnectionManagement")?;
        let current_db: String = conn_mgmt.funcall("current_db", ())?;
        if current_db != "default" {
            return Ok(false);
        }
    }
    
    let current_user_module = magnus::class::object().const_get::<_, RModule>("CurrentUser")?;
    let env: Value = request.funcall("env", ())?;
    
    let current_user_res: Result<Value, Error> = current_user_module.funcall("lookup_from_env", (env,));
    
    match current_user_res {
        Ok(current_user) => {
            if current_user.is_nil() {
                return Ok(false);
            }
            let is_admin: bool = current_user.funcall("admin?", ())?;
            if !is_admin {
                return Ok(false);
            }
            
            let custom_check: bool = rb_self.funcall("custom_admin_check", (request,))?;
            Ok(custom_check)
        }
        Err(_) => {
            Ok(false)
        }
    }
}

fn custom_admin_check(_rb_self: Value, _request: Value) -> Result<bool, Error> {
    Ok(true)
}

pub fn init() -> Result<(), Error> {
    let class = magnus::class::object().const_get::<_, RClass>("AdminConstraint")?;
    class.define_method("initialize", method!(initialize_admin_constraint, -1))?;
    class.define_method("matches?", method!(matches, 1))?;
    class.define_method("custom_admin_check", method!(custom_admin_check, 1))?;
    Ok(())
}
