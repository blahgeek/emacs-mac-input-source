use std::ffi::c_void;
use std::fmt::Display;

use crate::lisp_utils;

use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{Boolean, CFType, CFTypeID, OSStatus, TCFType, TCFTypeRef};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef, CFMutableDictionary};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::{declare_TCFType, impl_CFTypeDescription, impl_TCFType};

#[derive(Debug, Clone)]
pub struct OSStatusError(OSStatus);

impl Display for OSStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OSStatus error: {}", self.0)
    }
}

impl std::error::Error for OSStatusError {}

pub type Result<T> = std::result::Result<T, OSStatusError>;

fn wrap_osstatus(status: OSStatus) -> Result<()> {
    if status == 0 { Ok(()) } else { Err(OSStatusError(status)) }
}

type VecString = Vec<String>;

#[repr(C)]
pub struct __TISInputSource(c_void);

pub type TISInputSourceRef = *const __TISInputSource;

declare_TCFType!(TISInputSource, TISInputSourceRef);
impl_TCFType!(TISInputSource, TISInputSourceRef, TISInputSourceGetTypeId);
impl_CFTypeDescription!(TISInputSource);

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    static kTISPropertyInputSourceCategory: CFStringRef;  // -> CFStringRef
    static kTISPropertyInputSourceType: CFStringRef;  // -> CFStringRef
    static kTISPropertyInputSourceIsASCIICapable: CFStringRef;  // -> CFBooleanRef
    static kTISPropertyInputSourceIsEnableCapable: CFStringRef;  // -> CFBooleanRef
    static kTISPropertyInputSourceIsSelectCapable: CFStringRef;  // -> CFBooleanRef
    static kTISPropertyInputSourceIsEnabled: CFStringRef;  // -> CFBooleanRef
    static kTISPropertyInputSourceIsSelected: CFStringRef;  // -> CFBooleanRef
    static kTISPropertyInputSourceID: CFStringRef;  // -> CFStringRef
    static kTISPropertyBundleID: CFStringRef;  // -> CFStringRef
    static kTISPropertyInputModeID: CFStringRef;  // -> CFStringRef
    static kTISPropertyLocalizedName: CFStringRef;  // -> CFStringRef
    static kTISPropertyInputSourceLanguages: CFStringRef;  // -> CFStringRef

    fn TISInputSourceGetTypeId() -> CFTypeID;

    fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
    fn TISCopyCurrentKeyboardLayoutInputSource() -> TISInputSourceRef;
    fn TISCopyCurrentASCIICapableKeyboardInputSource() -> TISInputSourceRef;
    fn TISCopyCurrentASCIICapableKeyboardLayoutInputSource() -> TISInputSourceRef;
    fn TISCopyInputMethodKeyboardLayoutOverride() -> TISInputSourceRef;
    fn TISCopyInputSourceForLanguage(language: CFStringRef) -> TISInputSourceRef;

    fn TISGetInputSourceProperty(input_source: TISInputSourceRef, key: CFStringRef) -> *const c_void;

    fn TISCreateInputSourceList(props: CFDictionaryRef, include_all_installed: Boolean) -> CFArrayRef;
    fn TISCreateASCIICapableInputSourceList() -> CFArrayRef;

    fn TISSelectInputSource(source: TISInputSourceRef) -> OSStatus;
    fn TISDeselectInputSource(source: TISInputSourceRef) -> OSStatus;
    fn TISSetInputMethodKeyboardLayoutOverride(layout: TISInputSourceRef) -> OSStatus;
}

macro_rules! with_properties {
    ($cb:ident) => {
        $cb!(
            [ kTISPropertyInputSourceCategory, String, category ],
            [ kTISPropertyInputSourceType, String, type_ ],
            [ kTISPropertyInputSourceIsASCIICapable, bool, is_ascii_capable ],
            [ kTISPropertyInputSourceIsEnableCapable, bool, is_enable_capable ],
            [ kTISPropertyInputSourceIsSelectCapable, bool, is_select_capable ],
            [ kTISPropertyInputSourceIsEnabled, bool, is_enabled ],
            [ kTISPropertyInputSourceIsSelected, bool, is_selected ],
            [ kTISPropertyInputSourceID, String, id ],
            [ kTISPropertyBundleID, String, bundle_id ],
            [ kTISPropertyInputModeID, String, input_mode_id ],
            [ kTISPropertyLocalizedName, String, localized_name ],
            [ kTISPropertyInputSourceLanguages, VecString, languages ]
        );
    };
}

macro_rules! gen_cb_struct {
    ( $([ $key:ident, $type:ty, $field:ident ]),* ) => {
        #[derive(Debug, Default)]
        pub struct TISInputSourceProperties {
            $(
                pub $field : Option<$type>,
            )*
        }
    };
}

macro_rules! gen_to_dict_field {
    ( $self:tt, $result:ident, $key:ident, String, $field:ident ) => {
        if let Some(ref v) = $self.$field {
            $result.set(unsafe { CFString::wrap_under_create_rule($key) }.into_CFType(),
                        CFString::new(v.as_str()).into_CFType());
        }
    };
    ( $self:tt, $result:ident, $key:ident, bool, $field:ident ) => {
        if let Some(ref v) = $self.$field {
            $result.set(unsafe { CFString::wrap_under_create_rule($key) }.into_CFType(),
                        CFBoolean::from(*v).into_CFType());
        }
    };
    ( $self:tt, $result:ident, $key:ident, VecString, $field:ident ) => {
        // VecString not supported for now.
        // Only languages is VecString and it's not used for filtering anyway
    };
}

macro_rules! gen_to_dict {
    ($([ $key:ident, $type:ident, $field:ident ]),* ) => {
        fn to_dict(&self) -> CFDictionary {
            let mut result = CFMutableDictionary::<CFType, CFType>::new();
            $( gen_to_dict_field!(self, result, $key, $type, $field); )*
                result.to_immutable().into_untyped()
        }
    };
}

macro_rules! gen_into_lisp_field {
    // convert to plist
    // push value, then push key
    ( $self:tt, $env:ident, $result:ident, VecString, $field:ident ) => {
        $result = $env.cons(lisp_utils::vec_into_lisp($env, $self.$field)?, $result)?;
        $result = $env.cons(lisp_utils::property_name_to_lisp($env, stringify!($field))?, $result)?;
    };
    ( $self:tt, $env:ident, $result:ident, $type:ident, $field:ident ) => {
        $result = $env.cons($self.$field.into_lisp($env)?, $result)?;
        $result = $env.cons(lisp_utils::property_name_to_lisp($env, stringify!($field))?, $result)?;
    };
}

macro_rules! gen_into_lisp {
    ($([ $key:ident, $type:ident, $field:ident ]),* ) => {
        fn into_lisp(self, env: &emacs::Env) -> emacs::Result<emacs::Value<'_>> {
            let mut result = ().into_lisp(env)?;
            $( gen_into_lisp_field!(self, env, result, $type, $field); )*
            Ok(result)
        }
    }
}

with_properties!(gen_cb_struct);

impl TISInputSourceProperties {
    with_properties!(gen_to_dict);
}

impl emacs::IntoLisp<'_> for TISInputSourceProperties {
    with_properties!(gen_into_lisp);
}

impl TISInputSource {
    fn _wrap_create(source: TISInputSourceRef) -> Option<Self> {
        if source != std::ptr::null() {
            unsafe { Some(Self::wrap_under_create_rule(source)) }
        } else {
            None
        }
    }

    pub fn new_current_keyboard() -> Option<Self> {
        Self::_wrap_create(unsafe { TISCopyCurrentKeyboardInputSource() })
    }
    pub fn new_current_keyboard_layout() -> Option<Self> {
        Self::_wrap_create(unsafe { TISCopyCurrentKeyboardLayoutInputSource() })
    }
    pub fn new_current_ascii_capable_keyboard() -> Option<Self> {
        Self::_wrap_create(unsafe { TISCopyCurrentASCIICapableKeyboardInputSource() })
    }
    pub fn new_current_ascii_capable_keyboard_layout() -> Option<Self> {
        Self::_wrap_create(unsafe { TISCopyCurrentASCIICapableKeyboardLayoutInputSource() })
    }
    pub fn new_input_method_keyboard_layout_override() -> Option<Self> {
        Self::_wrap_create(unsafe { TISCopyInputMethodKeyboardLayoutOverride() })
    }
    pub fn new_for_language(language: &str) -> Option<Self> {
        Self::_wrap_create(unsafe {
            TISCopyInputSourceForLanguage(
                CFString::new(language).as_concrete_TypeRef())
        } )
    }

    fn _wrap_create_list(results: CFArrayRef) -> Vec<Self> {
        unsafe {
            if results == std::ptr::null() {
                Vec::new()
            } else {
                CFArray::<*const c_void>::wrap_under_create_rule(results)
                    .iter()
                    .map(|x| TISInputSource::wrap_under_get_rule(*x as TISInputSourceRef))
                    .collect()
            }
        }
    }

    pub fn new_list(filters: &TISInputSourceProperties, include_all_installed: bool) -> Vec<Self> {
        let filters_dict = filters.to_dict();
        unsafe {
            let results = TISCreateInputSourceList(filters_dict.as_concrete_TypeRef(),
                                                   include_all_installed as Boolean);
            Self::_wrap_create_list(results)
        }
    }

    pub fn new_ascii_capable_list() -> Vec<Self> {
        unsafe {
            Self::_wrap_create_list(TISCreateASCIICapableInputSourceList())
        }
    }

    fn get_property<T: TCFType>(&self, key: CFStringRef) -> Option<T> {
        unsafe {
            let res = TISGetInputSourceProperty(self.0, key);
            if res != std::ptr::null() {
                Some(T::wrap_under_get_rule(T::Ref::from_void_ptr(res)))
            } else {
                None
            }
        }
    }

    pub fn select(&self) -> Result<()> {
        wrap_osstatus( unsafe { TISSelectInputSource(self.0) } )
    }
    pub fn deselect(&self) -> Result<()> {
        wrap_osstatus( unsafe { TISDeselectInputSource(self.0) } )
    }
    pub fn set_keyboard_layout_override(&self) -> Result<()> {
        wrap_osstatus( unsafe { TISSetInputMethodKeyboardLayoutOverride(self.0) } )
    }
}

macro_rules! gen_get_properties_field {
    ( $self:tt, $result:ident, $key:ident, String, $field:ident ) => {
        $result.$field = $self.get_property::<CFString>(unsafe { $key })
            .map(|s| s.to_string());
    };
    ( $self:tt, $result:ident, $key:ident, bool, $field:ident ) => {
        $result.$field = $self.get_property::<CFBoolean>(unsafe { $key })
            .map(|s| s.into());
    };
    ( $self:tt, $result:ident, $key:ident, VecString, $field:ident ) => {
        $result.$field = $self.get_property::<CFArray::<*const c_void>>(unsafe { $key })
            .map(|v| v.iter()
                 .map(|x|
                      unsafe { CFString::wrap_under_get_rule(*x as CFStringRef) }
                      .to_string())
                 .collect());
    };
}

macro_rules! gen_get_properties {
    ($([ $key:ident, $type:ident, $field:ident ]),* ) => {
        pub fn get_properties(&self) -> Result<TISInputSourceProperties> {
            let mut result = TISInputSourceProperties::default();
            $( gen_get_properties_field!(self, result, $key, $type, $field); )*
            Ok(result)
        }
    }
}

impl TISInputSource {
    with_properties!(gen_get_properties);
}
