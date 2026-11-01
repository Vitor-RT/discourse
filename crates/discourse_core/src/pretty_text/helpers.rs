use magnus::{function, prelude::*, Error, Ruby, Value, RModule, RClass, Symbol, RArray};

fn t(key: String, opts: Value) -> Result<Value, Error> {
    let i18n = magnus::class::object().const_get::<_, RModule>("I18n")?;
    let full_key = format!("js.{}", key);
    let args = RArray::new();
    args.push(full_key)?;
    if !opts.is_nil() {
        args.push(opts)?;
    }
    i18n.funcall("t", (args.as_value(),))
}

fn avatar_template(username: Value) -> Result<String, Error> {
    if username.is_nil() { return Ok(String::new()); }
    let uname: String = username.funcall("to_s", ())?;
    let user_class = magnus::class::object().const_get::<_, RClass>("User")?;
    let user: Value = user_class.funcall("find_by", (Symbol::new("username_lower"), uname.to_lowercase()))?;
    if user.is_nil() { return Ok(String::new()); }

    let tmpl: String = user.funcall("avatar_template", ())?;
    let url_helper = magnus::class::object().const_get::<_, RModule>("UrlHelper")?;
    let abs: String = url_helper.funcall("absolute", (tmpl,))?;
    url_helper.funcall("schemaless", (abs,))
}

fn lookup_primary_user_group(username: Value) -> Result<String, Error> {
    if username.is_nil() { return Ok(String::new()); }
    let uname: String = username.funcall("to_s", ())?;
    let user_class = magnus::class::object().const_get::<_, RClass>("User")?;
    let user: Value = user_class.funcall("find_by", (Symbol::new("username_lower"), uname.to_lowercase()))?;
    if user.is_nil() { return Ok(String::new()); }

    let pg: Value = user.funcall("primary_group", ())?;
    if pg.is_nil() { return Ok(String::new()); }
    pg.funcall("name", ())
}

fn format_username(username: String) -> String {
    username
}

fn lookup_upload_urls(urls: Value) -> Result<Value, Error> {
    let proc = magnus::eval::<Value>(include_str!("helpers_eval1.rb"))?;
    proc.funcall("call", (urls,))
}

fn get_topic_info(topic_id: Value) -> Result<Value, Error> {
    let proc = magnus::eval::<Value>(include_str!("helpers_eval2.rb"))?;
    proc.funcall("call", (topic_id,))
}

fn hashtag_lookup(slug: Value, cooking_user_id: Value, types_in_priority_order: Value) -> Result<Value, Error> {
    let proc = magnus::eval::<Value>(include_str!("helpers_eval3.rb"))?;
    proc.funcall("call", (slug, cooking_user_id, types_in_priority_order))
}

fn get_current_user(user_id: Value) -> Result<Value, Error> {
    let proc = magnus::eval::<Value>(include_str!("helpers_eval4.rb"))?;
    proc.funcall("call", (user_id,))
}

pub fn init(_ruby: &Ruby, pt: RModule) -> Result<(), Error> {
    let helpers = pt.define_module("Helpers")?;
    helpers.const_set("TAG_HASHTAG_POSTFIX", "::tag")?;

    helpers.define_singleton_method("t", function!(t, 2))?;
    helpers.define_singleton_method("avatar_template", function!(avatar_template, 1))?;
    helpers.define_singleton_method("lookup_primary_user_group", function!(lookup_primary_user_group, 1))?;
    helpers.define_singleton_method("format_username", function!(format_username, 1))?;
    helpers.define_singleton_method("lookup_upload_urls", function!(lookup_upload_urls, 1))?;
    helpers.define_singleton_method("get_topic_info", function!(get_topic_info, 1))?;
    helpers.define_singleton_method("hashtag_lookup", function!(hashtag_lookup, 3))?;
    helpers.define_singleton_method("get_current_user", function!(get_current_user, 1))?;

    Ok(())
}
