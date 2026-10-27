//! Demonstração do Patina no navegador: texto estático, contador e campo.

use patina::prelude::*;

fn contador() -> View {
    let cliques = signal(0);

    el("button")
        .class("botao")
        .on(Click, move |_| cliques.update(|n| *n += 1))
        .text("Cliques: ")
        .child(cliques)
        .build()
}

fn saudacao() -> View {
    let nome = signal(String::from("mundo"));

    el("div")
        .class("campo")
        .child(
            el("label")
                .attr("for", "nome")
                .text("Seu nome"),
        )
        .child(
            el("input")
                .attr("id", "nome")
                .attr("value", "mundo")
                .on(Input, move |ev| nome.set(ev.value().unwrap_or_default().to_string())),
        )
        .child(el("p").text_with(move || format!("Olá, {}!", nome.get())))
        .build()
}

pub fn pagina() -> View {
    el("main")
        .class("patina-demo")
        .child(el("h1").text("Olá, Patina!"))
        .child(el("p").text("Esta página foi montada com uma árvore de views em Rust."))
        .child(contador())
        .child(saudacao())
        .build()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    patina::mount("#app", pagina())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagina_renderiza_estado_inicial() {
        let html = pagina().render_html();
        assert!(html.starts_with("<main class=\"patina-demo\"><h1>Olá, Patina!</h1>"));
        assert!(html.contains("<button class=\"botao\">Cliques: 0</button>"));
        assert!(html.contains("<p>Olá, mundo!</p>"));
    }
}
