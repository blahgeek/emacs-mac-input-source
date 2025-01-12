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

fn _input_source_list_to_lisp(env: &Env, sources: Vec<TISInputSource>) -> Result<Value<'_>> {
    env.list(
        &sources
            .into_iter()
            .map(|x| _input_source_to_lisp(env, Some(x)))
            .collect::<Result<Vec<Value<'_>>>>()?
    )
}

// Do not use defun(user_ptr) because that would convert the whole Option<...> into user_ptr
// let's implements Transfer and call into_lisp by ourselves

macro_rules! define_simple_new_input_source_method {
    ($name:ident) => {
        #[emacs::defun]
        fn $name(env: &Env) -> Result<Value<'_>> {
            _input_source_to_lisp(env, TISInputSource::$name())
        }
    };
}

define_simple_new_input_source_method!(new_current_keyboard);
define_simple_new_input_source_method!(new_current_keyboard_layout);
define_simple_new_input_source_method!(new_current_ascii_capable_keyboard);
define_simple_new_input_source_method!(new_current_ascii_capable_keyboard_layout);
define_simple_new_input_source_method!(new_input_method_keyboard_layout_override);

#[emacs::defun]
fn new_for_language(env: &Env, lang: String) -> Result<Value<'_>> {
    _input_source_to_lisp(env, TISInputSource::new_for_language(&lang))
}

#[emacs::defun]
fn new_list<'e>(env: &'e Env, id: Value<'e>, include_all_installed: Value<'e>) -> Result<Value<'e>> {
    // NOTE: bool does not implement FromLisp
    let include_all_installed = !include_all_installed.eq(().into_lisp(env)?);
    let mut props = TISInputSourceProperties::default();
    if !id.eq(().into_lisp(&env)?) {
        // id not null
        props.id = Some(String::from_lisp(id)?);
    }
    _input_source_list_to_lisp(
        env,
        TISInputSource::new_list(&props, include_all_installed)
    )
}

#[emacs::defun]
fn new_ascii_capable_list(env: &Env) -> Result<Value<'_>> {
    _input_source_list_to_lisp(env, TISInputSource::new_ascii_capable_list())
}


macro_rules! define_simple_input_source_get_or_action {
    ($name:ident, $result_type:ty) => {
        // Do not write `input_source: &TISInputSource` as param,
        // because `defun` macro would expect the value type to be RefCell (corresponding to defun(user_ptr))
        #[emacs::defun]
        fn $name(_: &Env, input_source_val: Value<'_>) -> Result<$result_type> {
            let input_source: &TISInputSource = FromLisp::from_lisp(input_source_val)?;
            Ok(input_source.$name()?)
        }
    };
}

define_simple_input_source_get_or_action!(get_properties, TISInputSourceProperties);
define_simple_input_source_get_or_action!(select, ());
define_simple_input_source_get_or_action!(deselect, ());
define_simple_input_source_get_or_action!(set_keyboard_layout_override, ());

