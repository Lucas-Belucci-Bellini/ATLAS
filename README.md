# ATLAS  — framework de frontend em Rust

> Versão inicial: `0.1.0` — **esqueleto experimental**, não um framework pronto para produção.
> Antes chamado de RustFront (nome de trabalho). Ver [ADR-0004](plans/decisions/ADR-0004-nome-patina.md).

ATLAS é o framework de frontend em Rust da família: uma superfície declarativa para interfaces web executadas no navegador por WebAssembly, com uma pequena árvore de views e um renderer DOM próprio. A intenção é controlar a arquitetura do framework, sem usar Leptos, Dioxus ou Yew como runtime subjacente.

Os nomes oficiais da API pública incluem `AtlasView`, `AtlasComponent`, `AtlasElementBuilder`, `AtlasSignal`, `AtlasEffect`, `AtlasShow`, `AtlasList`, os eventos `AtlasClick`/`AtlasInput` e os tipos HTTP `AtlasHttpClient`, `AtlasHttpResponse` e `AtlasApiEnvelope`. Os nomes genéricos e os aliases Patina continuam disponíveis por compatibilidade; o namespace de crates `patina-*` não é renomeado nesta etapa.

## Objetivos

- Construir interfaces declarativas usando estruturas Rust.
- Renderizar elementos e texto no DOM do navegador via WebAssembly.
- Evoluir para estado reativo, eventos, roteamento e ferramentas de desenvolvimento.
- Manter o núcleo independente do navegador sempre que isso for viável.
- Dar erros úteis, APIs pequenas e documentação em português.

## O que já existe 

- Workspace Cargo com `patina-reactive`, `patina-core`, `patina-dom` e a crate de entrada `patina`.
- Modelo `AtlasView` com texto (fixo ou dinâmico), fragmentos e elementos.
- Builder pequeno (`el`, `.class`, `.attr`, `.attr_with`, `.text`, `.text_with`, `.child`, `.on`) e o trait `AtlasComponent`.
- Signals, effects, `batch`, `untrack` e escopos com limpeza explícita.
- Eventos (`Click`, `Input`, `Submit`…) removidos automaticamente no `unmount`.
- Atualização localizada: quando um signal muda, só o nó de texto ou atributo afetado é tocado.
- Exemplo `examples/hello-web` com contador e campo de texto.
- Cliente HTTP/JSON exclusivo para WASM (`AtlasHttpClient`, `AtlasHttpResponse`, `AtlasApiEnvelope`), com timeout, rejeição de respostas não-2xx, parse de JSON e metadados de `x-request-id`; a demonstração inclui uma chamada opcional ao endpoint v1 do TITAN (verificação de navegador pendente).
- Testes nativos para o modelo de views e a reatividade, incluindo ausência de vazamentos.

## O que ainda não existe

Não há renderização condicional nem listas com chave, roteador, CLI própria, hot reload, SSR ou hidratação. Os listeners ainda não têm testes automatizados no navegador. Esses itens estão organizados no roadmap.

## Requisitos

- Rust e Cargo instalados via [rustup](https://rustup.rs/).
- Para gerar WebAssembly: `rustup target add wasm32-unknown-unknown`.
- Para empacotar: `cargo install wasm-pack`.

## Compilar/testar

Na raiz do projeto:

```bash
cargo test --workspace
cargo check --workspace
```

## Gerar a demonstração WebAssembly

Na raiz do projeto:

```bash
wasm-pack build examples/hello-web --target web
cd examples/hello-web
python -m http.server 8000
```

Abra `http://localhost:8000`. O `wasm-pack` grava o pacote em `examples/hello-web/pkg`.

## Exemplo da API

```rust
use patina::prelude::*;

fn pagina() -> AtlasView {
    el("main")
        .class("pagina")
        .child(el("h1").text("Olá, ATLAS!"))
        .child(el("p").text("UI web escrita em Rust."))
        .build()
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    patina::mount("#app", pagina())
}
```

`.child` aceita uma `AtlasView`, um builder ainda não finalizado, texto, um `AtlasSignal` ou um `AtlasComponent`.

### Estado e eventos

```rust
use patina::prelude::*;

fn contador() -> AtlasView {
    let cliques = signal(0);

    el("button")
        .on(AtlasClick, move |_| cliques.update(|n| *n += 1))
        .text("Cliques: ")
        .child(cliques) // só este nó de texto é atualizado
        .build()
}

fn saudacao() -> AtlasView {
    let nome = signal(String::from("mundo"));

    el("div")
        .child(el("input").on(AtlasInput, move |ev| nome.set(ev.value().unwrap_or_default().into())))
        .child(el("p").text_with(move || format!("Olá, {}!", nome.get())))
        .build()
}
```

`patina::mount` mantém a interface viva enquanto a página existir. Para desmontar, use `patina::mount_scoped("#app", || pagina())`, que devolve um `Mounted` com `unmount()`: ele remove os nós, os listeners e os effects.

## Estrutura

```text
crates/
  patina/          # crate de entrada: prelude, mount e mount_scoped
  patina-reactive/ # signals, effects e escopos, sem dependência do DOM
  patina-core/     # modelo de view, builders e eventos, sem dependência do DOM
  patina-dom/      # renderer reativo para o DOM em wasm32
examples/
  hello-web/     # demonstração (crate cdylib + index.html)
plans/           # produto, arquitetura, roadmap e decisões técnicas
```

## Princípios para contribuições

1. Não misturar runtime, roteamento e componentes numa única crate.
2. Manter o núcleo livre de dependências de navegador.
3. Ter testes para APIs públicas e regressões do renderer.
4. Evitar macros complexas antes de estabilizar o modelo de componentes.
5. Considerar acessibilidade e segurança desde o desenho da API.

## Referências técnicas

- [`wasm-bindgen`](https://docs.rs/wasm-bindgen/latest/wasm_bindgen/)
- [`web-sys`](https://docs.rs/web-sys/latest/web_sys/)
- [Guia oficial wasm-bindgen — web-sys](https://wasm-bindgen.github.io/wasm-bindgen/web-sys/index.html)

## Licença

MIT. Ver `LICENSE`.
