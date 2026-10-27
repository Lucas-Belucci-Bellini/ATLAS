# Escopo e status

Este pacote entrega documentação de planejamento e uma prova de conceito inicial. Não é uma release 1.0. A compilação e os testes precisam ser executados com Rust/Cargo no ambiente do usuário ou CI antes de considerar o código validado.

## Status em 2026-10-25

- `cargo test --workspace` e `cargo check --workspace` passam (Rust 1.99).
- `examples/hello-web` compila para `wasm32-unknown-unknown`.
- Signals, effects, escopos, eventos e atualização localizada do DOM implementados (Fase 2).

## Prioridade atual

1. Testes de navegador com `wasm-bindgen-test` para listeners e `unmount`.
2. Renderização condicional (`show`) e listas com chave.
3. CI com testes nativos e build wasm32.
