# Backlog inicial — Patina

As tarefas abaixo já podem virar issues. Marque-as concluídas apenas depois de executar os critérios de aceite.

## RF-001 — Validar ambiente e pipeline
**Prioridade:** P0

- Configurar CI com Rust stable.
- Executar `cargo fmt --all -- --check`, `cargo test --workspace` e `cargo check --workspace`.
- Adicionar build para `wasm32-unknown-unknown` e publicar o log do build.

**Aceite:** o pipeline executa em pull requests e falha quando qualquer etapa falha.

## RF-002 — Testes de browser para renderer
**Prioridade:** P0

- Instalar/configurar `wasm-bindgen-test`.
- Verificar texto, atributos, elementos aninhados, fragmentos e erro quando o alvo não existe.

**Aceite:** testes documentados e executáveis num navegador/headless runner.

## RF-003 — Definir contrato de componente
**Prioridade:** P1

- Propor uma API de componente que aceite propriedades tipadas.
- Manter builders e views testáveis nativamente.
- Documentar composição e erros de compilação comuns.

**Aceite:** duas telas de exemplo compartilham o mesmo componente sem duplicar a estrutura.

## RF-004 — Sinais reativos e efeitos
**Prioridade:** P1

- Definir `Signal<T>`, assinaturas, efeitos e regras de ciclo de vida.
- Desenhar atualização incremental antes de implementar.
- Cobrir unsubscribe e descarte de effects.

**Aceite:** alterar um signal atualiza somente os nós dependentes e não deixa callbacks órfãos.

## RF-005 — Eventos de DOM
**Prioridade:** P1

- Criar handlers para eventos comuns.
- Definir ownership e limpeza de callbacks `Closure`.
- Tratar listeners ao desmontar componente.

**Aceite:** handlers funcionam e são removidos na desmontagem.

## RF-006 — Roteador SPA
**Prioridade:** P2

- Suportar rotas estáticas, parâmetros e fallback 404.
- Definir comportamento de back/forward e deep links.

**Aceite:** exemplo com duas rotas passa por navegação interna e recarregamento direto da URL.
