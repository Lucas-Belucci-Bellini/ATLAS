# ADR-0002 — Árvore de views e montagem completa no MVP

- **Status:** aceito para o MVP
- **Data:** 2026-10-22

## Decisão

Modelar texto, fragmentos e elementos numa enumeração `View`. O renderer cria os nós recursivamente e substitui os filhos atuais do elemento-alvo.

## Motivo

É uma base simples de entender, testar e verificar antes de introduzir diffing e ciclo de vida.

## Limitações e próximo passo

O renderer não é incremental e pode recriar todo o conteúdo. Antes de adicionar estado reativo, desenhar identidade de nós, remoção de efeitos e limpeza de listeners.
