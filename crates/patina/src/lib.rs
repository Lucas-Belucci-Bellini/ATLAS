//! ATLAS — Rust UI framework for browser/WASM applications.
//! Atlas-prefixed public names are the official API. Patina-prefixed aliases
//! remain available for compatibility with earlier releases.

pub use patina_core::{el, ev, Attribute, Component, ElementBuilder, Event, EventType, View};
pub use patina_reactive::{
    batch, create_root, effect, on_cleanup, signal, untrack, Effect, Root, Signal,
};

/// Official Atlas name for an HTML attribute descriptor.
pub use patina_core::Attribute as AtlasAttribute;
/// Official Atlas name for DOM event values.
pub use patina_core::Event as AtlasEvent;
/// Official Atlas name for DOM event kinds.
pub use patina_core::EventType as AtlasEventType;
/// Official Atlas name for a reactive ownership root.
pub use patina_reactive::Root as AtlasRoot;

/// Official Atlas name for View.
pub type AtlasView = View;
/// Official Atlas name for Component.
pub use patina_core::Component as AtlasComponent;
/// Official Atlas name for ElementBuilder.
pub use patina_core::ElementBuilder as AtlasElementBuilder;
/// Official Atlas name for Signal.
pub type AtlasSignal<T> = Signal<T>;
/// Official Atlas name for Effect.
pub type AtlasEffect = Effect;

/// Compatibility aliases for the previous public branding.
pub type PatinaView = AtlasView;
pub use patina_core::Attribute as PatinaAttribute;
pub use patina_core::Component as PatinaComponent;
pub use patina_core::ElementBuilder as PatinaElementBuilder;
pub use patina_core::Event as PatinaEvent;
pub use patina_core::EventType as PatinaEventType;
pub type PatinaSignal<T> = AtlasSignal<T>;
pub type PatinaEffect = AtlasEffect;
pub use patina_reactive::Root as PatinaRoot;

/// Browser HTTP/JSON client, compiled only for WebAssembly.
#[cfg(target_arch = "wasm32")]
pub mod http;

/// Everything commonly needed to build a page.
pub mod prelude {
    pub use crate::{
        AtlasAttribute, AtlasComponent, AtlasEffect, AtlasElement, AtlasElementBuilder,
        AtlasEvent, AtlasEventType, AtlasRoot, AtlasSignal, AtlasView, PatinaAttribute,
        PatinaComponent, PatinaEffect, PatinaElementBuilder, PatinaEvent, PatinaEventType,
        PatinaRoot, PatinaSignal, PatinaView,
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
