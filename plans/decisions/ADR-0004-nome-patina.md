# ADR-0004 — Nome do projeto: Patina

## Status

Aceito em 2026-10-25.

## Contexto

"RustFront" era um nome de trabalho. Ele descreve a tecnologia, mas não é memorável nem distinguível de outros projetos "Rust + front".

## Decisão

O projeto passa a se chamar **Patina**. Pátina é a camada que se forma na superfície do metal: Rust é o metal, Patina é a superfície que o usuário vê.

- Crates: `patina` (entrada, com `prelude` e `mount`), `patina-core`, `patina-dom`.
- O demo saiu de `patina-dom` para `examples/hello-web`, uma crate `cdylib` própria; `patina-dom` volta a ser só biblioteca.
- `mount` passa a receber um seletor CSS (`"#app"`) em vez de um id.
- Identidade visual: fundo ferro `#121416`, cobre `#E08A5A`, verdete `#6CC4A8`; Bricolage Grotesque, IBM Plex Sans e IBM Plex Mono.

## Consequências

- Antes de publicar, verificar se `patina`, `patina-core` e `patina-dom` estão livres no crates.io. Alternativa com o mesmo conceito: **Verdete**.
- A pasta do repositório ainda se chama `rustfront-framework`; renomeá-la é uma decisão separada.
