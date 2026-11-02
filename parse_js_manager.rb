methods = File.read("lib/plugin/js_manager.rb").scan(/def (?:self\.)?([a-zA-Z0-9_\?!]+)/).flatten
rust_methods = methods.map do |m|
  rust_name = m.tr("?", "").tr("!", "")
  "    pub fn #{rust_name}() { unimplemented!() }"
end

rust_code = <<~RUST
pub struct JsManager;

impl JsManager {
#{rust_methods.join("\n")}
}
RUST

File.write("crates/discourse_core/src/plugin/js_manager.rs", rust_code)
