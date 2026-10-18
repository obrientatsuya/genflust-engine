# ADR 0002 — Superfície pública inicial (~60 funções)

**Status:** aceito · **Fontes:** [§2.2](../../ENGINE_VISION2.md#22-minimalismo), [§2.5](../../ENGINE_VISION2.md#25-luau--rust), [§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets), [§4.7](../../ENGINE_VISION2.md#47-compatibilidade-com-o-que-a-ia-já-sabe), [§5](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli), [R16](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)

## Contexto

A API é o que o agente aprende. Pequena, previsível e estável vale mais que
completa. Orçamento: ~60 funções; adicionar exige remover outra ou justificar
([R16](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)). Módulos opcionais não podem poluir a API básica.

## Decisão

1. **Inventário inicial** ([§5](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli)): `gfx` 6 · `input` 3 · `snd` 1 · `ent` 3 + 4
   propostas · `scene`/`msg`/`evt` 3 propostas · `phys` 2 · `juice` 6 ·
   globais 2 ≈ **29 hoje** + propostas ≈ 36. Folga deliberada até ~60.
2. **Uma responsabilidade por função**: `gfx.rect` preenche, `gfx.rectb`
   contorna — nenhum flag mágico. Sem sinônimos (um nome por coisa).
3. **Callbacks**: `load()`/`update(dt)`/`draw()` (LÖVE). Controlador
   (fase 3, [§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets)): `load(e)` `update(e,dt)` `on_message` `on_event` `unload`.
4. **Opção não polui**: `juice`, `phys`, `snd` e `audio` vivem em módulos
   opcionais; o L0 (`llms.txt`) lista só o que está ativo ([§3.5](../../ENGINE_VISION2.md#35-plugins-tudo-que-quiser-carregar)).
5. **Estabilidade**: quebrar = camada de compatibilidade + versão
   ([R16](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)/[R20](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)); nunca quebrar o que o agente já aprendeu ([§4.7](../../ENGINE_VISION2.md#47-compatibilidade-com-o-que-a-ia-já-sabe)).

## Consequências

- Orçamento auditável no L0 (1 linha por função, ≤3k tokens).
- Função nova exige: remover outra ou justificar + `.d.luau` + docs + exemplo
  rodando com `--expect` ([§5](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli)).
- Implementado na fase 1: `gfx.*` (6), `ent.*` (3), `rnd` (1) — o resto segue
  o roadmap.

## Em aberto

Assinaturas finais das `[p]` (scene/msg/evt/attach/detach): só congelam após
validação no jogo de referência (fase 3).
