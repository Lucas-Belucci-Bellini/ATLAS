# ADR-0001 — Runtime inicial em WebAssembly sobre web-sys

- **Status:** aceito para a prova de conceito
- **Data:** 2026-10-22

## Contexto

O objetivo é construir e controlar um framework próprio em Rust para interfaces web, sem depender de outro framework de UI como runtime base.

## Decisão

Usar `wasm-bindgen` e `web-sys` como fronteira para APIs do navegador. Manter o modelo de views em uma crate independente do browser.

## Consequências

- Controle sobre a API pública do framework.
- Mais código a implementar e manter, especialmente reatividade, eventos e DOM diffing.
- A prova de conceito deve limitar-se à montagem completa da árvore; otimização vem depois de testes e medição.
