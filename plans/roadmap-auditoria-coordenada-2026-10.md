# Roadmap técnico — ATLAS (auditoria coordenada, outubro de 2026)

> Estado desta revisão: inspeção estática de manifests, README e fontes principais via GitHub. **Nenhum comando de teste, build WASM ou teste de navegador foi executado.**
>
> Branch de trabalho: `roadmap/coordinated-audit-2026-10`. A branch `main` não foi alterada.

## 1. Papel do ATLAS no ecossistema

ATLAS é o frontend Rust/WASM independente: modelo de views, runtime reativo e renderer DOM. O nome público do repositório é ATLAS, mas a identidade técnica e os crates atualmente são Patina/`patina-*`. Há um ADR aceito em 2026-10-25 escolhendo Patina; não renomear crates automaticamente. Uma decisão posterior pode separar nome do repositório do nome do framework.

## 2. Estado de código observado

- Workspace Cargo com resolver 2, edition 2021, licença MIT e crates `patina-reactive`, `patina-core`, `patina-dom`, `patina` e exemplo `hello-web`.
- `patina-reactive`: runtime por thread, signals, effects, batching, tracking/untracking e escopos/cleanup; IDs têm generation para detectar handles descartados.
- `patina-core`: modelo `View`, atributos e texto dinâmicos, builders, eventos e componentes sem depender diretamente do DOM.
- Importante: o código inspecionado em `patina-core` já contém os tipos `Show`, `List`, `Key` e APIs relacionadas; `patina-dom/src/browser.rs` importa e usa esses tipos. Isso contradiz a seção do README que ainda declara renderização condicional e listas com chave como ausentes. A implementação existe ao menos em parte, mas sua semântica e cobertura precisam ser verificadas antes de atualizar a documentação.
- `patina-dom`: renderer condicionado a `wasm32`; monta seletor CSS, registra eventos e `Mounted::unmount` descarta escopos e limpa nós/listeners.
- `patina`: facade/prelude e `mount`/ `mount_scoped` para WASM.
- `examples/hello-web`: contador, input e saudação reativa; há teste nativo para HTML inicial.
- `Cargo.lock` existe. Os caminhos de workflow `.github/workflows/ci.yml` e `.github/workflows/rust.yml` não foram encontrados; confirmar se existe workflow com outro nome.
- README ainda declara ausentes router, CLI, hot reload, SSR/hidratação e testes automatizados de listeners no navegador. Esses são candidatos de roadmap, não funcionalidades presentes.

## 3. Lacunas e riscos para validar

1. Baseline nativo: format, tests, clippy e check com lockfile.
2. Baseline WASM: compilar `wasm32-unknown-unknown`, gerar o exemplo e verificar se a saída é executável no navegador.
3. Auditar atualização condicional e listas keyed: criação, remoção, reordenação, identidade de itens e descarte de escopos.
4. Testar ciclos e dependências do runtime reativo, reentrância de effects, writes durante effects e cleanup exatamente uma vez.
5. Testar falhas de montagem e desmontagem: seletor inexistente, DOM APIs com erro, listeners e effects sem vazamento.
6. Criar testes de navegador reais; testes nativos do modelo não validam integração com DOM/browser.
7. Auditar escaping de texto/atributos e tratamento de nomes/valores de atributos para prevenir injeção HTML.
8. Decidir política para SSR/hidratação somente depois de estabilizar o modelo de view e semântica de keys.
9. Conferir docs/roadmap: o README aponta para `plans/`, e o ADR-0004 já documenta o nome Patina e pendências de crates.io.

## 4. Roadmap por fases

### Fase A — Baseline e correção de documentação (P0)
- Rodar `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` e `cargo check --workspace --locked`.
- Instalar alvo WASM e rodar `cargo check --workspace --target wasm32-unknown-unknown`; gerar o demo com `wasm-pack build examples/hello-web --target web`.
- Confirmar toolchain mínima real: o manifesto não declara `rust-version`; definir MSRV suportada após testar.
- Atualizar inventário de features somente depois de validar Show/List no renderer.
- Verificar se arquivos gerados `pkg/` são ignorados e não entram por engano no versionamento.
- Configurar CI para nativo e WASM, separando testes de navegador que exigem runtime/driver.

**Aceite:** comandos registrados; CI reproduzível; README corresponde ao código verificado; MSRV declarada e justificada.

### Fase B — Semântica de reatividade e renderer (P1)
- Especificar garantias de `Signal`, `effect`, `batch`, `untrack`, `create_root` e cleanup.
- Testar que effects antigos deixam de observar signals depois de reexecução.
- Testar `show`: transição true/false, `otherwise`, criação do ramo apenas na mudança e descarte de efeitos/handlers do ramo anterior.
- Testar listas keyed: preservar identidade ao reordenar, remover somente itens removidos, criar apenas itens novos e liberar escopos.
- Testar listeners adicionados/removidos e desmontagem repetida/ownership do `Mounted`.
- Rever panics em signals após descarte e documentar se são parte intencional do contrato ou se a API precisa de retorno fallible.

**Aceite:** testes determinísticos para transições, identidade e cleanup; sem vazamentos reproduzíveis.

### Fase C — Qualidade de navegador e acessibilidade (P1)
- Adicionar suite browser automation para click/input/change/submit, preventDefault, atributos dinâmicos, show/list e unmount.
- Adicionar testes de escaping e conteúdo malicioso em texto/atributos.
- Documentar suporte de eventos, elementos de formulário, atributos booleanos e propriedades DOM.
- Definir mínimos de acessibilidade nos exemplos: labels, foco, teclado e semântica HTML.

**Aceite:** bugs de integração browser capturados por CI e exemplos demonstram práticas seguras.

### Fase D — Router e ferramentas (P2)
- Especificar router com rotas tipadas, navegação History API, parâmetros, query strings, fallback e cleanup.
- Criar CLI somente depois de definir estrutura de aplicação e fluxo de build WASM.
- Planejar hot reload como ferramenta externa ao runtime principal; não misturar no núcleo reativo.
- Publicação de crates só após checar disponibilidade dos nomes `patina*` no crates.io e confirmar licença/metadados.

**Aceite:** ADRs para router/CLI; semântica de navegação testada; nomes de crates verificados antes de publicar.

### Fase E — SSR e hidratação (P3, decisão condicionada)
- Avaliar custo/benefício depois do router e do modelo de view estáveis.
- Definir como o HTML do servidor corresponde à árvore client-side e como evitar render duplicado/handlers duplicados.
- Criar ADR antes de implementar; não prometer SSR enquanto runtime usa estado por thread e tipos `Rc/RefCell` sem desenho explícito para servidor.

## 5. Ordem recomendada de execução

1. Baseline nativo + WASM.
2. Validar Show/List e corrigir divergência README/código.
3. Especificação e testes do runtime/cleanup.
4. Browser automation, escaping e acessibilidade.
5. Router.
6. CLI/hot reload.
7. Avaliação de SSR/hidratação por ADR.

## 6. Integração com AETHER e TITAN

- ATLAS deve consumir APIs HTTP sem depender de crates backend.
- Exemplos de integração podem apontar para AETHER ou TITAN, mas não devem criar dependência de build nem acoplamento de runtime.
- Definir um padrão de envelope de erro no frontend como adaptador de cliente, sem impor que todos os servidores usem o mesmo crate.

## 7. Fora de escopo por enquanto

- SSR/hidratação antes de estabilizar semântica do renderer.
- Macros complexas antes de consolidar View/Component.
- Reescrever o runtime ou trocar de framework reativo sem benchmarks e ADR.
- Renomear Patina/crates sem nova decisão registrada.

## 8. Critério de conclusão da auditoria

Cada feature deve ser classificada como implementada e testada, implementada sem cobertura suficiente, parcial, ausente ou não verificada. A presença de tipos `Show`/`List` é evidência de implementação, não prova de correção em todos os casos.
