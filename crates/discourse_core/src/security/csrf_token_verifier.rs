use magnus::{method, prelude::*, Error, Value, RClass, RModule};

fn call(rb_self: Value, env: Value) -> Result<(), Error> {
    let action_dispatch = magnus::class::object().const_get::<_, RModule>("ActionDispatch")?;
    let request_class = action_dispatch.const_get::<_, RClass>("Request")?;

    let env_dup: Value = env.funcall("dup", ())?;
    let request: Value = request_class.new_instance((env_dup,))?;

    rb_self.funcall::<_, _, Value>("instance_variable_set", ("@request", request))?;

    let verified: bool = rb_self.funcall("verified_request?", ())?;
    if !verified {
        let invalid_csrf_token_class = rb_self.class().const_get::<_, magnus::ExceptionClass>("InvalidCSRFToken")?;
        return Err(Error::new(invalid_csrf_token_class, "Invalid CSRF Token"));
    }

    Ok(())
}

pub fn init() -> Result<(), Error> {
    let class = magnus::class::object().const_get::<_, RClass>("CSRFTokenVerifier")?;
    class.define_method("call", method!(call, 1))?;
    Ok(())
}
