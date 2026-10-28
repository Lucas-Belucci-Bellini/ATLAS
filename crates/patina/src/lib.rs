//! ATLAS — Rust UI framework for browser/WASM applications.
//! PatinaView, PatinaComponent, PatinaElementBuilder and PatinaSignal are
//! framework-branded aliases. Existing short names remain available.

pub use patina_core::{el, ev, Attribute, Component, ElementBuilder, Event, EventType, View};
pub use patina_reactive::{
    batch, create_root, effect, on_cleanup, signal, untrack, Effect, Root, Signal,
};

/// Framework-branded alias for View.
pub type PatinaView = View;
/// Framework-branded alias for Component.
pub use patina_core::Component as PatinaComponent;
/// Framework-branded alias for ElementBuilder.
pub use patina_core::ElementBuilder as PatinaElementBuilder;
/// Framework-branded alias for Signal.
pub type PatinaSignal<T> = Signal<T>;
/// Framework-branded alias for Effect.
pub type PatinaEffect = Effect;

/// Browser HTTP/JSON client, compiled only for WebAssembly.
#[cfg(target_arch = "wasm32")]
pub mod http;

/// Everything commonly needed to build a page.
pub mod prelude {
    pub use crate::{
        PatinaComponent, PatinaEffect, PatinaElementBuilder, PatinaSignal, PatinaView,
    };
    pub use patina_core::ev::{
        Blur, Change, Click, DoubleClick, Focus, Input, KeyDown, KeyUp, Submit,
    };
    pub use patina_core::{el, Component, ElementBuilder, Event, View};
    pub use patina_reactive::{batch, effect, on_cleanup, signal, untrack, Signal};
}

#[cfg(target_arch = "wasm32")]
pub use patina_dom::Mounted;

/// Mounts a view into the first element matching the CSS selector target.
#[cfg(target_arch = "wasm32")]
pub fn mount(target: &str, view: impl Into<View>) -> Result<(), wasm_bindgen::JsValue> {
    let view = view.into();
    patina_dom::mount(target, move || view).map(|_| ())
}

/// Mounts a view built inside its mount scope and returns a handle for unmount.
#[cfg(target_arch = "wasm32")]
pub fn mount_scoped(
    target: &str,
    build: impl FnOnce() -> View,
) -> Result<Mounted, wasm_bindgen::JsValue> {
    patina_dom::mount(target, build)
}
