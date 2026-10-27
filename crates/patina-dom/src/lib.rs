//! Renderer DOM para navegador. A integração só é compilada para `wasm32`.
//!
//! Texto e atributos dinâmicos viram effects: quando um signal muda, só o nó
//! de texto ou o atributo afetado é atualizado. Listeners são removidos no
//! `Mounted::unmount`.

#[cfg(target_arch = "wasm32")]
mod browser;

#[cfg(target_arch = "wasm32")]
pub use browser::{mount, Mounted};

/// Marcador simples para permitir `cargo check --workspace` em hosts nativos.
/// A função `mount` só está disponível para builds `wasm32-unknown-unknown`.
#[cfg(not(target_arch = "wasm32"))]
pub fn browser_renderer_requires_wasm() -> &'static str {
    "Compile com --target wasm32-unknown-unknown para habilitar o renderer DOM."
}
