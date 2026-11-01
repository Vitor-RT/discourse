pub mod core_bundle;
pub mod helpers;

use lazy_static::lazy_static;
use regex::Regex;
use magnus::{function, prelude::*, Error, Ruby, Value, RModule, RArray};

lazy_static! {
    pub static ref DANGEROUS_BIDI_CHARACTERS: Vec<&'static str> = vec![
        "\u{202A}", "\u{202B}", "\u{202C}", "\u{202D}", "\u{202E}",
        "\u{2066}", "\u{2067}", "\u{2068}", "\u{2069}",
    ];
    pub static ref DANGEROUS_BIDI_REGEXP: Regex = {
        let pattern = DANGEROUS_BIDI_CHARACTERS.join("|");
        Regex::new(&pattern).unwrap()
    };
    pub static ref VIMEO_PLAYER_PATH: Regex = Regex::new(r"\A/video/(?P<id>\d+)/?\z").unwrap();
    pub static ref VIMEO_UNLISTED_HASH: Regex = Regex::new(r"\A[a-zA-Z0-9]+\z").unwrap();
}

pub const BLOCKED_HOTLINKED_SRC_ATTR: &str = "data-blocked-hotlinked-src";
pub const BLOCKED_HOTLINKED_SRCSET_ATTR: &str = "data-blocked-hotlinked-srcset";
pub const VIMEO_PLAYER_HOST: &str = "player.vimeo.com";

fn app_root() -> Result<Value, Error> {
    magnus::eval("Rails.root")
}

fn method_missing(_method_name: String, _args: RArray) -> Result<Value, Error> {
    Ok(magnus::eval("nil").unwrap())
}

#[magnus::init(name = "pretty_text")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let pt = ruby.define_module("PrettyText")?;

    // Constants
    let bidi_chars = RArray::new();
    for c in DANGEROUS_BIDI_CHARACTERS.iter() { bidi_chars.push(*c)?; }
    pt.const_set("DANGEROUS_BIDI_CHARACTERS", bidi_chars)?;
    pt.const_set("BLOCKED_HOTLINKED_SRC_ATTR", BLOCKED_HOTLINKED_SRC_ATTR)?;
    pt.const_set("BLOCKED_HOTLINKED_SRCSET_ATTR", BLOCKED_HOTLINKED_SRCSET_ATTR)?;
    pt.const_set("VIMEO_PLAYER_HOST", VIMEO_PLAYER_HOST)?;

    core_bundle::init(ruby, pt)?;
    helpers::init(ruby, pt)?;

    pt.define_singleton_method("app_root", function!(app_root, 0))?;

    // Include full fallback logic
    magnus::eval::<Value>(include_str!("mod_eval.rb"))?;

    Ok(())
}
