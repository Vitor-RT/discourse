use magnus::{function, method, prelude::*, Error, Ruby, Value, class, Class};
use super::PrettyText;

#[magnus::init(name = "pretty_text")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("PrettyText")?;

    Ok(())
}
