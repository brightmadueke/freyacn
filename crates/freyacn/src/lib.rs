#![doc = include_str!("../README.md")]
#[doc = ""]
#[doc = include_str!("../API.md")]
extern crate self as freyacn;

pub mod cn;
pub mod components;
pub mod extensions;
pub mod stylesheet;
pub mod styling;
pub mod theme;
pub mod utils;
pub mod variants;

pub mod prelude {
    #[doc(inline)]
    pub use crate::cn;
    #[doc(inline)]
    pub use crate::components::*;
    #[doc(inline)]
    pub use crate::extensions::style::{StyleExt, *};
    #[doc(inline)]
    pub use crate::theme::{
        Colors, Theme, get_cn_theme_or_default, use_cn_theme, use_init_cn_theme,
        use_provide_cn_theme,
    };
    #[doc(inline)]
    pub use crate::utils::*;
    #[doc(inline)]
    pub use crate::variants::Variant;
    #[doc(inline)]
    pub use freya::icons::lucide::{image, radio, timer, *};
    #[doc(inline)]
    pub use freya::prelude::{Button as ButtonPrimitive, StyleExt as _, *};
    #[doc(inline)]
    pub use freya_core::element::EventHandlerType;
    #[doc(inline)]
    pub use freya_core::events::name::EventName;
    #[doc(inline)]
    pub use freyacn_macros::*;
    #[doc(inline)]
    pub use rustc_hash::FxHashMap;
}
