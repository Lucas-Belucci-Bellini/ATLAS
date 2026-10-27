# Plano de produto — Patina

## Problema

Desenvolver uma biblioteca/framework própria de frontend em Rust, com uma API consistente e controle sobre as decisões fundamentais do runtime. A saída inicial é uma interface web compilada para WebAssembly.

## Usuários-alvo

- Desenvolvedores Rust que desejam escrever a lógica de UI em Rust.
- Pessoas que constroem ferramentas internas, dashboards e aplicações SPA.
- O próprio projeto como laboratório para aprender runtime, DOM, WebAssembly e APIs públicas de framework.

## Proposta de valor

- API Rust declarativa em vez de manipulação manual dispersa do DOM.
- Núcleo pequeno e separação clara entre árvore de views e integração com navegador.
- Evolução incremental, testável e extensível.

## Fora do escopo da versão 0.1

- Ser substituto maduro de React, Leptos ou Dioxus.
- SSR, hidratação, mobile nativo e desktop.
- Um compilador de templates próprio.
- Compatibilidade retroativa garantida antes da versão 1.0.

## Critérios de sucesso do MVP

1. Compor uma página por código Rust.
2. Compilar o renderer para `wasm32-unknown-unknown`.
3. Montar texto, elementos, atributos e fragmentos no DOM.
4. Testar o modelo de views sem navegador.
5. Documentar limites conhecidos e passos para reproduzir o exemplo.
