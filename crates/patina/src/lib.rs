//! ATLAS — Rust UI framework for browser/WASM applications.
//! Atlas-prefixed public names are the official API. Patina-prefixed aliases
//! remain available for compatibility with earlier releases.

pub use patina_core::{el, ev, Attribute, Component, ElementBuilder, Event, EventType, View};
pub use patina_reactive::{
    batch, create_root, effect, on_cleanup, signal, untrack, Effect, Root, Signal,
};

/// Official Atlas name for a DOM blur event.
pub use patina_core::ev::Blur as AtlasBlur;
/// Official Atlas name for a DOM change event.
pub use patina_core::ev::Change as AtlasChange;
/// Official Atlas name for a DOM click event.
pub use patina_core::ev::Click as AtlasClick;
/// Official Atlas name for a DOM double-click event.
pub use patina_core::ev::DoubleClick as AtlasDoubleClick;
/// Official Atlas name for a DOM focus event.
pub use patina_core::ev::Focus as AtlasFocus;
/// Official Atlas name for a DOM input event.
pub use patina_core::ev::Input as AtlasInput;
/// Official Atlas name for a DOM key-down event.
pub use patina_core::ev::KeyDown as AtlasKeyDown;
/// Official Atlas name for a DOM key-up event.
pub use patina_core::ev::KeyUp as AtlasKeyUp;
/// Official Atlas name for a DOM submit event.
pub use patina_core::ev::Submit as AtlasSubmit;
/// Official Atlas name for an HTML attribute descriptor.
pub use patina_core::Attribute as AtlasAttribute;
/// Official Atlas name for DOM event values.
pub use patina_core::Event as AtlasEvent;
/// Official Atlas name for DOM event kinds.
pub use patina_core::EventType as AtlasEventType;
/// Official Atlas names for core view and rendering types.
pub use patina_core::{
    AttrValue as AtlasAttrValue, DynText as AtlasDynText, Key as AtlasKey, List as AtlasList,
    ListItem as AtlasListItem, Listener as AtlasListener, Show as AtlasShow,
};
/// Official Atlas name for a reactive ownership owner.
pub use patina_reactive::Owner as AtlasOwner;
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
    #[cfg(target_arch = "wasm32")]
    pub use crate::http::{AtlasApiEnvelope, AtlasHttpClient, AtlasHttpResponse};
    #[cfg(target_arch = "wasm32")]
    pub use crate::AtlasMounted;

    pub use crate::{
        AtlasAttrValue, AtlasAttribute, AtlasBlur, AtlasChange, AtlasClick, AtlasComponent,
        AtlasDoubleClick, AtlasDynText, AtlasEffect, AtlasElementBuilder, AtlasEvent,
        AtlasEventType, AtlasFocus, AtlasInput, AtlasKey, AtlasKeyDown, AtlasKeyUp, AtlasList,
        AtlasListItem, AtlasListener, AtlasOwner, AtlasRoot, AtlasShow, AtlasSignal, AtlasSubmit,
        AtlasView, PatinaAttribute, PatinaComponent, PatinaEffect, PatinaElementBuilder,
        PatinaEvent, PatinaEventType, PatinaRoot, PatinaSignal, PatinaView,
    };
    pub use patina_core::ev::{
        Blur, Change, Click, DoubleClick, Focus, Input, KeyDown, KeyUp, Submit,
    };
    pub use patina_core::{el, Component, ElementBuilder, Event, View};
    pub use patina_reactive::{batch, effect, on_cleanup, signal, untrack, Signal};
}

#[cfg(target_arch = "wasm32")]
pub use patina_dom::Mounted;
/// Official Atlas name for a mounted DOM view.
#[cfg(target_arch = "wasm32")]
pub use patina_dom::Mounted as AtlasMounted;

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
