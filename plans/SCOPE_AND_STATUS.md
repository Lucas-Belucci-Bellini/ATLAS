# Escopo e status

ATLAS continua sendo um framework experimental em versão `0.1.0`, não uma release 1.0 nem um framework pronto para produção. Os pacotes Cargo mantêm o namespace `patina-*` durante a transição de compatibilidade.

## Status verificado em 2026-10-30

- CI nativo: formatação, check, testes do workspace e Clippy.
- Compilação para `wasm32-unknown-unknown`.
- Smoke test de runtime em Chromium para o exemplo `hello-web`.
- O teste de navegador cobre renderização inicial, interações reativas, desmontagem da montagem escopada, remoção do DOM e evento tardio em elemento destacado sem erros de página/console.
- Auditoria automatizada de dependências integrada ao CI.
- Contrato HTTP versionado com TITAN verificado por workflow de integração; isso valida o endpoint de contrato, não uma integração frontend/backend pronta para produção.

Consulte os checks do commit/PR correspondente para confirmar o resultado de cada execução. Não trate uma execução anterior como garantia de checks futuros.

## Implementado

- Views declarativas e builder.
- Signals, effects, eventos DOM e atualização localizada.
- Montagem normal e escopada com cleanup explícito.
- Cliente HTTP/JSON para WASM com timeout, tratamento de respostas não-2xx, parse JSON e metadados de request ID.

## Ainda fora do escopo implementado

- Roteador SPA, listas com chave, CLI, dev server/HMR, SSR e hidratação.
- Matriz ampla de testes de navegador, acessibilidade e compatibilidade entre navegadores.
- Garantia de estabilidade de API 1.0 ou prontidão para produção.

## Prioridades atuais

1. Expandir testes de browser para renderer, atributos, composição e erros.
2. Implementar renderização condicional e listas com chave.
3. Desenhar roteamento, formulários e acessibilidade antes de ampliar a API.
