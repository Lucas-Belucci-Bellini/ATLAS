# Roadmap — Patina

## Fase 0 — Fundação (agora)

- [x] Workspace e crates `core`/`dom`.
- [x] Modelo inicial de views e builder.
- [x] Renderer de montagem completa.
- [x] Exemplo HTML e decisões iniciais.
- [x] Executar `cargo test`, `cargo check` e build wasm32 numa máquina com toolchain.
- [ ] Configurar CI para testes nativos e build WebAssembly.

**Saída:** prova de conceito reproduzível.

## Fase 1 — Componentes e runtime mínimo

- [x] Trait/abstração de componente e composição de componentes (`Component`, versão inicial).
- [ ] Tratamento consistente de atributos e propriedades.
- [ ] Mensagens de erro e APIs de diagnóstico.
- [ ] Testes de renderização no navegador (por exemplo, `wasm-bindgen-test`).

**Saída:** páginas compostas a partir de unidades reutilizáveis.

## Fase 2 — Estado e eventos

- [x] Signals e subscriptions (`patina-reactive`).
- [x] Effects com limpeza explícita (`on_cleanup`, `Root::dispose`).
- [x] Listeners `click`, `input`, `submit` e remoção automática ao desmontar.
- [x] Atualização localizada do DOM para texto e atributos.
- [x] Testes de ciclo de vida e de ausência de signals/effects órfãos (nativos).
- [ ] Testes de listeners no navegador (`wasm-bindgen-test`).
- [ ] Renderização condicional e listas com chave.

**Saída:** UI interativa.

## Fase 3 — Aplicações reais

- [ ] Roteador SPA, links e parâmetros.
- [ ] Camada de fetch, estados loading/error/success e cancelamento.
- [ ] Formulários, validação e acessibilidade.
- [ ] Tratamento de erros de componentes.

**Saída:** primeiro dashboard demonstrativo.

## Fase 4 — Ferramentas e DX

- [ ] CLI para criar projeto e executar build.
- [ ] Dev server e recarga automática; HMR só se tiver desenho viável.
- [ ] Templates de projeto e exemplos.
- [ ] CI, benchmarks e documentação de migração.

**Saída:** fluxo de desenvolvimento repetível.

## Fase 5 — Estabilidade

- [ ] Auditoria de dependências e tamanho do bundle.
- [ ] Testes de acessibilidade e navegadores.
- [ ] Política de versionamento semântico.
- [ ] Decisão fundamentada sobre SSR/hidratação.
- [ ] Critérios de lançamento `1.0` documentados.
