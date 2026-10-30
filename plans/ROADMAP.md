# Roadmap — ATLAS (pacotes atuais: patina-*)

## Fase 0 — Fundação

- [x] Workspace e crates `patina-core`, `patina-dom`, `patina-reactive` e facade `patina`.
- [x] Modelo inicial de views e builder.
- [x] Renderer de montagem completa.
- [x] Exemplo `examples/hello-web`.
- [x] CI para testes nativos e compilação WebAssembly.
- [x] Auditoria automatizada de dependências.

**Saída:** prova de conceito reproduzível em CI.

## Fase 1 — Componentes e runtime mínimo

- [x] Trait/abstração de componente e composição.
- [x] Builder de elementos, atributos e propriedades atualmente suportados.
- [x] API de montagem e montagem escopada com handle explícito.
- [ ] Ampliar mensagens de erro e APIs de diagnóstico.
- [x] Smoke test de navegador Chromium para o exemplo público.

**Saída:** páginas compostas a partir de unidades reutilizáveis, com validação inicial no navegador.

## Fase 2 — Estado e eventos

- [x] Signals e subscriptions (`patina-reactive`).
- [x] Effects com limpeza explícita (`on_cleanup`, `Root::dispose`).
- [x] Listeners `click`, `input`, `submit` e remoção automática ao desmontar.
- [x] Atualização localizada do DOM para texto e atributos.
- [x] Testes nativos de lifecycle e descarte de signals/effects.
- [x] Smoke test Chromium para interação reativa, desmontagem e evento tardio em elemento destacado.
- [ ] Expandir a matriz de testes de browser para atributos, elementos aninhados, fragmentos e falhas de montagem.
- [ ] Renderização condicional (`show`) e listas com chave.

**Saída:** UI interativa com cobertura inicial de lifecycle no navegador.

## Fase 3 — Aplicações reais

- [ ] Roteador SPA, links e parâmetros.
- [ ] Camada de fetch, estados loading/error/success e cancelamento.
- [ ] Formulários, validação e acessibilidade.
- [ ] Tratamento de erros de componentes.

**Saída:** primeiro dashboard demonstrativo.

## Fase 4 — Ferramentas e DX

- [ ] CLI para criar projeto e executar build.
- [ ] Dev server e recarga automática; HMR só se tiver desenho viável.
- [x] Exemplo de projeto e workflow de CI reproduzível.
- [ ] Benchmarks e documentação de migração.

**Saída:** fluxo de desenvolvimento repetível.

## Fase 5 — Estabilidade

- [x] Auditoria automatizada de dependências integrada ao CI.
- [ ] Auditoria de tamanho do bundle.
- [ ] Testes de acessibilidade e matriz ampliada de navegadores.
- [ ] Política de versionamento semântico.
- [ ] Decisão fundamentada sobre SSR/hidratação.
- [ ] Critérios de lançamento `1.0` documentados.

**Nota:** CI verde e smoke test Chromium não equivalem a suporte amplo de navegadores nem a prontidão para produção.
