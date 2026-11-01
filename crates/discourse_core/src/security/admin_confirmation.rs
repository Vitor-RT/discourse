use magnus::{method, prelude::*, Error, Value, RClass, RModule};

fn create_confirmation(rb_self: Value) -> Result<(), Error> {
    let performed_by: Value = rb_self.funcall("performed_by", ())?;
    let target_user: Value = rb_self.funcall("target_user", ())?;

    let guardian_class = magnus::class::object().const_get::<_, RClass>("Guardian")?;
    let guardian: Value = guardian_class.new_instance((performed_by,))?;
    guardian.funcall::<_, _, Value>("ensure_can_grant_admin!", (target_user,))?;

    let secure_random = magnus::class::object().const_get::<_, RModule>("SecureRandom")?;
    let token: String = secure_random.funcall("hex", ())?;
    rb_self.funcall::<_, _, Value>("token=", (token.clone(),))?;

    let discourse = magnus::class::object().const_get::<_, RModule>("Discourse")?;
    let redis: Value = discourse.funcall("redis", ())?;

    let target_user_id: i64 = target_user.funcall("id", ())?;
    let key1 = format!("admin-confirmation:{}", target_user_id);
    let hours_3: i64 = 3 * 3600; // 3 hours in seconds
    redis.funcall::<_, _, Value>("setex", (key1, hours_3, token.clone()))?;

    let performed_by_id: i64 = performed_by.funcall("id", ())?;

    let hash = magnus::RHash::new();
    hash.aset(magnus::Symbol::new("target_user_id"), target_user_id)?;
    hash.aset(magnus::Symbol::new("performed_by"), performed_by_id)?;
    let payload_json: String = hash.funcall("to_json", ())?;

    let key2 = format!("admin-confirmation-token:{}", token);
    redis.funcall::<_, _, Value>("setex", (key2, hours_3, payload_json))?;

    let jobs = magnus::class::object().const_get::<_, RModule>("Jobs")?;
    let performed_by_email: String = performed_by.funcall("email", ())?;
    let target_user_email: String = target_user.funcall("email", ())?;
    let target_user_username: String = target_user.funcall("username", ())?;

    let job_args = magnus::RHash::new();
    job_args.aset(magnus::Symbol::new("to_address"), performed_by_email)?;
    job_args.aset(magnus::Symbol::new("target_email"), target_user_email)?;
    job_args.aset(magnus::Symbol::new("target_username"), target_user_username)?;
    job_args.aset(magnus::Symbol::new("token"), token)?;

    jobs.funcall::<_, _, Value>("enqueue", (magnus::Symbol::new("admin_confirmation_email"), job_args))?;

    Ok(())
}

fn email_confirmed(rb_self: Value) -> Result<(), Error> {
    let performed_by: Value = rb_self.funcall("performed_by", ())?;
    let target_user: Value = rb_self.funcall("target_user", ())?;

    let guardian_class = magnus::class::object().const_get::<_, RClass>("Guardian")?;
    let guardian: Value = guardian_class.new_instance((performed_by,))?;
    guardian.funcall::<_, _, Value>("ensure_can_grant_admin!", (target_user,))?;

    target_user.funcall::<_, _, Value>("grant_admin!", ())?;

    let staff_action_logger_class = magnus::class::object().const_get::<_, RClass>("StaffActionLogger")?;
    let staff_action_logger: Value = staff_action_logger_class.new_instance((performed_by,))?;
    staff_action_logger.funcall::<_, _, Value>("log_grant_admin", (target_user,))?;

    let token: String = rb_self.funcall("token", ())?;
    let target_user_id: i64 = target_user.funcall("id", ())?;

    let discourse = magnus::class::object().const_get::<_, RModule>("Discourse")?;
    let redis: Value = discourse.funcall("redis", ())?;

    let key1 = format!("admin-confirmation:{}", target_user_id);
    redis.funcall::<_, _, Value>("del", (key1,))?;

    let key2 = format!("admin-confirmation-token:{}", token);
    redis.funcall::<_, _, Value>("del", (key2,))?;

    Ok(())
}

fn exists_for(_class: Value, user_id: Value) -> Result<bool, Error> {
    let discourse = magnus::class::object().const_get::<_, RModule>("Discourse")?;
    let redis: Value = discourse.funcall("redis", ())?;

    let user_id_i64: i64 = user_id.funcall("to_i", ())?;
    let key = format!("admin-confirmation:{}", user_id_i64);
    let exists: bool = redis.funcall("exists?", (key,))?;
    Ok(exists)
}

fn find_by_code(class: Value, token: String) -> Result<Value, Error> {
    let discourse = magnus::class::object().const_get::<_, RModule>("Discourse")?;
    let redis: Value = discourse.funcall("redis", ())?;
    let key = format!("admin-confirmation-token:{}", token);
    let json_val: Value = redis.funcall("get", (key,))?;

    if json_val.is_nil() {
        let nil_val: Value = magnus::eval("nil").unwrap();
        return Ok(nil_val);
    }

    let json_str: String = json_val.funcall("to_s", ())?;
    let json_module = magnus::class::object().const_get::<_, RModule>("JSON")?;
    let parsed: magnus::RHash = json_module.funcall("parse", (json_str,))?;

    let target_user_id_val: Value = parsed.get("target_user_id").unwrap();
    let target_user_id: i64 = target_user_id_val.funcall("to_i", ())?;

    let performed_by_id_val: Value = parsed.get("performed_by").unwrap();
    let performed_by_id: i64 = performed_by_id_val.funcall("to_i", ())?;

    let user_class = magnus::class::object().const_get::<_, RClass>("User")?;
    let target_user: Value = user_class.funcall("find", (target_user_id,))?;
    let performed_by: Value = user_class.funcall("find", (performed_by_id,))?;

    let ac: Value = class.funcall("new", (target_user, performed_by))?;
    ac.funcall::<_, _, Value>("token=", (token,))?;

    Ok(ac)
}

pub fn init() -> Result<(), Error> {
    let class = magnus::class::object().const_get::<_, RClass>("AdminConfirmation")?;
    class.define_method("create_confirmation", method!(create_confirmation, 0))?;
    class.define_method("email_confirmed!", method!(email_confirmed, 0))?;

    let singleton = class.singleton_class()?;
    singleton.define_method("exists_for?", method!(exists_for, 1))?;
    singleton.define_method("find_by_code", method!(find_by_code, 1))?;

    Ok(())
}
