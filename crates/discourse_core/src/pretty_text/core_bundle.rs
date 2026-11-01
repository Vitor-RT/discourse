use magnus::{prelude::*, Error, Ruby, Value, RModule, RArray};

pub fn init(_ruby: &Ruby, pt: RModule) -> Result<(), Error> {
    let cb = pt.define_module("CoreBundle")?;

    let modules = RArray::new();
    let mods = [
        "deprecation-workflow",
        "lib/avatar-utils",
        "lib/case-converter",
        "lib/escape",
        "lib/get-url",
        "lib/object",
        "loader",
        "static/markdown-it/features"
    ];
    for m in mods.iter() {
        modules.push(*m)?;
    }
    cb.const_set("DISCOURSE_MODULES", modules)?;

    let bundle_val: Value = magnus::eval(include_str!("core_bundle.rb"))?;
    cb.const_set("BUNDLE", bundle_val)?;

    Ok(())
}
