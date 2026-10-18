# ADR 0003 — Escopo do primeiro jogo (referência, fase 3)

**Status:** aceito · **Fontes:** [§9](../../ENGINE_VISION2.md#9-roadmap), [§10](../../ENGINE_VISION2.md#10-backlog-e-organização-gratuito-minimalista), [§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets)

## Contexto

A arquitetura (entidades compostas, controladores, relações, mensagens) só se
prova com um jogo completo feito pelo fluxo da engine — antes de ampliar
plugins e editores.

## Decisão

1. **Cenário jogável (fase 3, [§9](../../ENGINE_VISION2.md#9-roadmap))**: Player e NPC compartilham movimento;
   espada independente equipa/solta; coleta abre porta via `relay`; troca de
   cena limpa recursos; Speaker opcional acompanha o Player.
2. **Meios obrigatórios para demonstrar**: `cycle`, `test --expect`,
   `llms.txt`/AGENTS mínimos, controlador único por entidade, mensagens e
   eventos ordenados, attach/detach, presets mínimos (`player` `npc` `static`
   `speaker` `empty` `relay` `spawner`), troca de cena.
3. **Arte por placeholder até o humano desenhar** ([§4.8](../../ENGINE_VISION2.md#48-arte-o-humano-cria-o-agente-usa)): o jogo roda e é
   testável do minuto 1; nomes ausentes viram caixa + rótulo e entram em
   `ART_TODO`.
4. **Aceite** ([§9](../../ENGINE_VISION2.md#9-roadmap)): agente cria e corrige pelo CLI usando placeholders;
   testes verificam comportamento e ciclo de vida.

## Exclusões do MVP (adiado — [Fora do MVP, AGENTS §10](../../AGENTS.md#10-escopo-e-fases-9-11-13))

- Múltiplos controladores por entidade (compor funções Luau em ordem
  explícita, [§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets)).
- Empilhamento/streaming de cenas; sistema de ossos.
- Plugins nativos dinâmicos (dylib), WASM, networking.
- Backend GPU de raster (GPU só apresenta na fase 2+).
- Editor de cenas/inspetor (fase 9); tilemaps, state machines e diálogos no
  editor.

## Consequências

- Fases 4–6 só ganham escopo que o jogo de referência demonstrar.
- Presets são composição de componentes, nunca subclasses ([§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets)).
