# Backlog — ATLAS (pacotes atuais: patina-*)

As tarefas abaixo preservam os IDs históricos para não quebrar referências. Status deve refletir evidência no código e nos workflows, não apenas intenção.

## RF-001 — Validar ambiente e pipeline
**Prioridade:** P0 · **Status:** concluído

- CI com Rust stable para formatação, check, testes e Clippy.
- Build para `wasm32-unknown-unknown`.
- Auditoria automatizada de dependências.

**Evidência:** workflows `Rust and WASM CI` e `Dependency Security Audit`. Consultar os resultados por SHA para cada PR.

## RF-002 — Testes de browser para renderer
**Prioridade:** P0 · **Status:** parcial

- Smoke test Playwright/Chromium do exemplo `hello-web`.
- Cobrir renderização inicial, interações reativas, desmontagem e evento tardio em elemento destacado.
- Expandir para atributos, elementos aninhados, fragmentos, alvo inexistente e erros de montagem.

**Aceite final:** matriz de browser/headless reproduzível que cubra as estruturas e falhas relevantes do renderer. O smoke test atual não satisfaz sozinho toda essa matriz.

## RF-003 — Definir contrato de componente
**Prioridade:** P1 · **Status:** parcial

- API inicial de componentes, builders e views existe.
- Continuar refinando propriedades tipadas, composição e documentação de erros.

**Aceite final:** duas telas de exemplo compartilham componentes sem duplicar estrutura e com contrato público documentado.

## RF-004 — Sinais reativos e efeitos
**Prioridade:** P1 · **Status:** implementação inicial concluída

- Signals, subscriptions, effects e descarte explícito implementados em `patina-reactive`.
- Atualização localizada de texto/atributos implementada.
- Continuar ampliando testes de regressão e casos de dependência.

**Aceite de evolução:** mudanças atualizam apenas os nós dependentes e não deixam callbacks órfãos.

## RF-005 — Eventos de DOM
**Prioridade:** P1 · **Status:** implementação inicial concluída

- Handlers de eventos comuns implementados.
- Ownership e cleanup dos listeners ligados à montagem escopada.
- Smoke test Chromium cobre desmontagem e evento tardio.

**Aceite de evolução:** ampliar matriz de eventos e navegadores.

## RF-006 — Roteador SPA
**Prioridade:** P2 · **Status:** não iniciado

- Suportar rotas estáticas, parâmetros e fallback 404.
- Definir comportamento de back/forward e deep links.

**Aceite:** exemplo com duas rotas passa por navegação interna e recarregamento direto da URL.
