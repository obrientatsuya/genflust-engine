# ADR 0004 — Decisões pendentes (hipóteses a provar)

**Status:** em aberto · **Fontes:** [§3.6-A/B/C](../../ENGINE_VISION2.md#36-performance-extremamente-veloz), [§6.4](../../ENGINE_VISION2.md#64-implementação)

Cada hipótese tem **experimento** e **critério de decisão** — nenhuma presume
o resultado. Não congelar antes de medir ([R19](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)).

## H1. Ponte Rust↔Luau: buffers SoA vs FFI por chamada

- **Hipótese:** buffers contíguos SoA (tipo `buffer` do Luau) batem as metas
  de hot path e o agente continua vendo só a API simples ([§3.6-A](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)). Alternativa
  em uso na fase 1: FFI por chamada (simples, sem hot path ainda).
- **Experimento:** jogo de referência com ~1000 entidades; medir FFI/frame,
  alocações e fps headless.
- **Critério:** FFI <200/frame · 0 alocações no hot path · headless >10k fps
  ([§3.6](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)). Se FFI por chamada já cumprir, adiar buffers sem culpa.

## H2. Matemática determinística: `libm` vs ponto fixo

- **Hipótese:** `libm` (Rust puro) dá hashes idênticos Linux/Windows com custo
  aceitável; ponto fixo é o plano B ([§3.6-C](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)).
- **Experimento:** benchmark com trigonometria e física simples; comparar
  golden hashes nos dois SOs + ms/frame.
- **Critério:** hashes idênticos e dentro do orçamento de frame; senão, ponto
  fixo nos módulos físicos. Sem fast-math em qualquer caso ([R7](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)).

## H3. `luau-jit`/`@native` por função quente

- **Hipótese:** geração nativa dá 1,5–2,5x só onde importa, com interpretador
  como fallback ([§3.6-B](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)).
- **Experimento:** `engine profile` no jogo de referência; marcar a função
  mais quente com `@native` e comparar.
- **Critério:** ganho ≥1,5x medido antes de ligar por padrão; se não justificar,
  continua desligado ([R19](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)).

## H4. GC Luau com orçamento no fim do frame

- **Hipótese:** passo incremental de GC no fim do frame mantém pausa <0,5ms
  ([§3.6](../../ENGINE_VISION2.md#36-performance-extremamente-veloz), [§3.6-B](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)).
- **Experimento:** medir pausa de GC por frame no jogo de referência.
- **Critério:** pausa <0,5ms/frame; se estourar, reavaliar padrões de
  alocação do Luau (lints do `check`).

## H5. UI do editor: egui vs dogfooding em Luau

- **Hipótese:** egui é mais rápido de construir e custa só no editor;
  dogfooding prova a API mas atrasa ([§6.4](../../ENGINE_VISION2.md#64-implementação), "decisão em aberto").
- **Experimento:** prototipar o editor de pixel art mínimo (lápis + salvar)
  nas duas opções antes da fase 7.
- **Critério:** tempo até editar/salvar um sprite funcionando, custo do
  binário do jogo (<5MB, editor separado) e se a API Luau aguenta a UI.
  Recomendação atual do doc: egui no MVP — decidir com o dado, não antes.

## H6. Mold/lld como linker de dev no Linux

- **Hipótese:** mold encurta link em dev sem afetar Windows ([§3.6-G](../../ENGINE_VISION2.md#36-performance-extremamente-veloz)).
- **Experimento:** linkar o workspace com/sem mold na máquina do amigo.
- **Critério:** ganho de tempo no link dev; opcional — nunca no CI de golden
  hashes (não muda o binário, mas não arriscar variância).
