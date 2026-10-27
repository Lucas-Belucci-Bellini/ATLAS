# Arquitetura — Patina

## Fluxo principal

```text
Código Rust da aplicação
        ↓
View / ElementBuilder (patina-core) ── lê ──► Signals (patina-reactive)
        ↓                                          │
Árvore de views (estática + nós dinâmicos)         │ notifica
        ↓                                          ▼
Renderer DOM (patina-dom, wasm32) ◄── effects por nó de texto/atributo
        ↓
DOM do navegador
```

## Crates

### `patina-reactive`

Responsabilidades: `signal`, `effect`, `batch`, `untrack` e escopos (`create_root`, `on_cleanup`, `Root::dispose`). Runtime por thread com slots geracionais: um `Signal` é um identificador `Copy`, e usar um signal descartado gera um panic com mensagem clara. Não depende de Web APIs nem de `patina-core`.

### `patina-core`

Responsabilidades: representar `Text`, `Dynamic`, `Fragment` e `Element`; armazenar atributos (estáticos ou dinâmicos) e listeners; fornecer builders e o trait `Component`. O evento entregue aos listeners (`Event`) guarda o evento nativo de forma opaca, então o core continua sem importar `web_sys`. `View::render_html` serializa o estado atual para testes; não é SSR.

### `patina-dom`

Responsabilidades: traduzir a árvore para nós reais do DOM dentro de um escopo reativo. Cada texto ou atributo dinâmico vira um effect que atualiza só aquele nó. Listeners são registrados com `addEventListener` e removidos no `Mounted::unmount`. Ainda não há reconciliação de listas nem renderização condicional de subárvores.

### `patina`

Crate de entrada: `prelude`, `mount` (vive enquanto a página viver) e `mount_scoped` (devolve `Mounted`).

## Camadas planejadas

- `patina-router`: rotas client-side, parâmetros e navegação.
- `patina-fetch`: chamadas HTTP, cancelamento e estados de carregamento.
- `patina-cli`: scaffolding, dev server e comando de build.
- `patina-ssr`: renderização no servidor e hidratação — somente após estabilidade do CSR.

Essas crates são propostas, não fazem parte da implementação atual.

## Regras de projeto

1. Core não deve importar `web_sys`.
2. O renderer deve propagar falhas com `Result`, sem `unwrap` em caminhos de runtime.
3. Nunca inserir conteúdo de usuário como HTML não sanitizado por conveniência; texto deve virar `Text`.
4. IDs, eventos e referências DOM deverão ter ciclo de vida explícito antes de implementar reatividade.
5. APIs públicas devem ter exemplos e testes antes de serem marcadas como estáveis.

## Riscos técnicos

- Criar DOM na mão aumenta o trabalho de eventos, formulários, foco, acessibilidade e diffing.
- WebAssembly não significa automaticamente bundle menor ou aplicação mais rápida; medir antes de prometer desempenho.
- SSR/hidratação exigem consistência de serialização e identidade dos nós; não incluir cedo demais.
- Dependências `web-sys` usam features por API: manter a lista mínima e revisar ao adicionar métodos.
