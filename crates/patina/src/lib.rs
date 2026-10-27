//! Patina — a superfície do seu Rust.
//!
//! Crate de entrada: reexporta o modelo de views de `patina-core`, a
//! reatividade de `patina-reactive` e, em `wasm32`, o renderer de `patina-dom`.
//!
//! ```
//! use patina::prelude::*;
//!
//! fn contador() -> View {
//!     let cliques = signal(0);
//!
//!     el("button")
//!         .on(Click, move |_| cliques.update(|n| *n += 1))
//!         .text("Cliques: ")
//!         .child(cliques)
//!         .build()
//! }
//!
//! assert_eq!(contador().render_html(), "<button>Cliques: 0</button>");
//! ```

pub use patina_core::{ev, el, Attribute, Component, ElementBuilder, Event, EventType, View};
pub use patina_reactive::{batch, create_root, effect, on_cleanup, signal, untrack, Effect, Root, Signal};

/// Tudo o que uma página costuma precisar: `use patina::prelude::*;`.
pub mod prelude {
    pub use patina_core::ev::{Blur, Change, Click, DoubleClick, Focus, Input, KeyDown, KeyUp, Submit};
    pub use patina_core::{el, Component, ElementBuilder, Event, View};
    pub use patina_reactive::{batch, effect, on_cleanup, signal, untrack, Signal};
}

#[cfg(target_arch = "wasm32")]
pub use patina_dom::Mounted;

/// Monta `view` no primeiro elemento que casa com o seletor CSS `target`,
/// substituindo o conteúdo atual. A interface vive enquanto a página viver;
/// para desmontar depois, use [`mount_scoped`]. Erros também vão ao console.
#[cfg(target_arch = "wasm32")]
pub fn mount(target: &str, view: impl Into<View>) -> Result<(), wasm_bindgen::JsValue> {
    let view = view.into();
    patina_dom::mount(target, move || view).map(|_| ())
}

/// Como [`mount`], mas constrói a árvore dentro do escopo da montagem e
/// devolve um [`Mounted`] para `unmount()`.
#[cfg(target_arch = "wasm32")]
pub fn mount_scoped(target: &str, build: impl FnOnce() -> View) -> Result<Mounted, wasm_bindgen::JsValue> {
    patina_dom::mount(target, build)
}
