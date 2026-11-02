methods = File.read("lib/plugin/instance.rb").scan(/def (?:self\.)?([a-zA-Z0-9_\?!]+)/).flatten
rust_methods = methods.map do |m|
  rust_name = m.tr("?", "").tr("!", "")
  "    pub fn #{rust_name}() { unimplemented!() }"
end

rust_code = <<~RUST
pub struct CustomEmoji;
impl CustomEmoji {
    pub fn cache_key() { unimplemented!() }
    pub fn emojis() { unimplemented!() }
    pub fn clear_cache() { unimplemented!() }
    pub fn register() { unimplemented!() }
    pub fn unregister() { unimplemented!() }
    pub fn translations() { unimplemented!() }
    pub fn translate() { unimplemented!() }
}

pub struct Instance;

impl Instance {
#{rust_methods.drop(7).join("\n")}
}
RUST

File.write("crates/discourse_core/src/plugin/instance.rs", rust_code)
