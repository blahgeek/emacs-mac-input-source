pub mod lisp_utils;
pub mod tis;  // text_input_source

use emacs::{Env, IntoLisp, Result, Transfer, Value, FromLisp};
use tis::{TISInputSource, TISInputSourceProperties};

emacs::plugin_is_GPL_compatible!();

// prefix ends with "-", so the actual name has "--" in the middle
#[emacs::module(name="mac-input-source-dyn", defun_prefix="mac-input-source-")]
fn init(env: &Env) -> Result<Value<'_>> {
    env.message("Loaded mac-input-source dynamic library!")
}

impl Transfer for TISInputSource {
    fn type_name() -> &'static str {
        "mac-input-source--TISInputSource"
    }
}

fn _input_source_to_lisp(env: &Env, source: Option<TISInputSource>) -> Result<Value<'_>> {
    match source {
        None => ().into_lisp(env),
        Some(s) => Box::new(s).into_lisp(env)
    }
}

// Do not use defun(user_ptr) because that would convert the whole Option<...> into user_ptr
// let's implements Transfer and call into_lisp by ourselves
#[emacs::defun]
fn new_current_keyboard(env: &Env) -> Result<Value<'_>> {
    _input_source_to_lisp(env, TISInputSource::new_current_keyboard())
}

// Do not write `input_source: &TISInputSource` as param,
// because `defun` macro would expect the value type to be RefCell (corresponding to defun(user_ptr))
#[emacs::defun]
fn get_properties(_: &Env, input_source_val: Value<'_>) -> Result<TISInputSourceProperties> {
    let input_source: &TISInputSource = FromLisp::from_lisp(input_source_val)?;
    Ok(input_source.get_properties())
}

