# ADR-0003 — Não incluir SSR na versão inicial

- **Status:** aceito provisoriamente
- **Data:** 2026-10-22

## Decisão

A primeira linha de desenvolvimento é client-side rendering (CSR) em WebAssembly. SSR e hidratação não entram na versão 0.1.

## Motivo

SSR exige um renderizador HTML de servidor e um protocolo claro de hidratação. Isso aumentaria a superfície de bugs antes de o modelo de componentes e o ciclo de vida estarem estáveis.

## Reavaliação

Reavaliar quando componentes, estado, identidade de nós e roteador tiverem testes estáveis.
