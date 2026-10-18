# ADR 0001 — Contratos determinísticos (tick, mutação, replay)

**Status:** aceito · **Fontes:** [§3.2](../../ENGINE_VISION2.md#32-fluxo-de-um-frame), [§3.6](../../ENGINE_VISION2.md#36-performance-extremamente-veloz), [§3.9](../../ENGINE_VISION2.md#39-comunicação-mensagens-dirigidas-e-eventos), [§4.3](../../ENGINE_VISION2.md#43-verificação-provar-que-funciona), [R7/R8/R9/R11](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)

## Contexto

Replay, testes e golden hashes exigem que o mesmo jogo, com a mesma seed e os
mesmos inputs, produza a mesma saída em qualquer SO. Ordem de execução e
fronteira entre mutação e observação precisam ser parte do contrato.

## Decisão

1. **Tick com ordem fixa** ([§3.2](../../ENGINE_VISION2.md#32-fluxo-de-um-frame)): barreira estrutural → mensagens do tick
   anterior → input → `update(dt)` + controladores → systems Rust →
   eventos/estado observável → `draw()` → raster → framebuffer → PNG|hash.
2. **Barreiras** ([R8](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)): criar/remover entidade, trocar componente/controlador
   ou cena só muda o mundo no início do próximo tick; `ent.new` reserva ID no
   tick e ativa na barreira; montagem usa o ID pendente na mesma fila.
3. **`draw()` e observadores nunca mutam a simulação** ([R9](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)): hash do estado
   é idêntico com observação ligada ou desligada.
4. **Determinismo numérico**: timestep fixo (1/60), RNG semeado (xoshiro256**),
   sem relógio de parede na simulação, ordem de iteração estável (Vec/índice).
5. **Condições de repetição de um replay** ([§3.9](../../ENGINE_VISION2.md#39-comunicação-mensagens-dirigidas-e-eventos)): mesma versão de código,
   assets e configuração; mesma seed; inputs gravados. Replay que não reproduz
   = erro de contrato, não "quase igual".

## Consequências

- Alterar a ordem do tick é quebra de contrato (versionar, [R20](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)).
- Qualquer consulta de observação deve ser read-only por construção.
- Golden hashes Linux+Windows são o teste do determinismo (fase 4; hoje a
  garantia local é `crates/cli/tests/determinism.rs`).

## Em aberto

Escolha da matemática determinística (`libm` vs ponto fixo): ADR 0004.
