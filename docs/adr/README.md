# ADRs — decisões da engine

Fonte de verdade: [ENGINE_VISION2.md](../../ENGINE_VISION2.md) (`§N` = seção). Um ADR registra uma decisão; a **matriz de escopo** abaixo separa **contratado** de **hipótese**. Em conflito, o doc vence.

| ADR | Assunto | Status |
|---|---|---|
| [0001](0001-contratos-deterministicos.md) | Tick, mutação/observação e condições de replay | Aceito ([§3.2](../../ENGINE_VISION2.md#32-fluxo-de-um-frame), [§4.3](../../ENGINE_VISION2.md#43-verificação-provar-que-funciona)) |
| [0002](0002-superficie-publica.md) | Superfície pública inicial (~60 funções) | Aceito ([§2.2](../../ENGINE_VISION2.md#22-minimalismo), [§5](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli), [R16](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança)) |
| [0003](0003-escopo-jogo-referencia.md) | Escopo do primeiro jogo (fase 3) | Aceito ([§9](../../ENGINE_VISION2.md#9-roadmap), [§10](../../ENGINE_VISION2.md#10-backlog-e-organização-gratuito-minimalista)) |
| [0004](0004-decisoes-pendentes.md) | Hipóteses com experimento e critério | Em aberto |

## Matriz de escopo

| Assunto | Decisão vigente | Status |
|---|---|---|
| Ordem do tick (8 fases, barreira, entrega no próximo tick) | [§3.2](../../ENGINE_VISION2.md#32-fluxo-de-um-frame) | **Contratado** |
| Timestep fixo 60Hz, RNG semeado, inputs gravados, golden hashes nos 2 SOs | [§3.6](../../ENGINE_VISION2.md#36-performance-extremamente-veloz), [§4.3](../../ENGINE_VISION2.md#43-verificação-provar-que-funciona), [R7](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança) | **Contratado** |
| Observação (Live View/log/MCP) nunca altera a simulação | [§3.9](../../ENGINE_VISION2.md#39-comunicação-mensagens-dirigidas-e-eventos), [R9](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança) | **Contratado** |
| Identidade única `ent`, componentes opcionais, 1 controlador/entidade | [§3.7](../../ENGINE_VISION2.md#37-modelo-de-mundo-cenas-e-entidades-compostas)–[§3.8](../../ENGINE_VISION2.md#38-controladores-reutilizáveis-e-presets) | **Contratado** |
| Cena/TOML, comportamento/Luau, pixels/PNG; CLI e editor com o mesmo carregador | [R15](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança), [§3.7](../../ENGINE_VISION2.md#37-modelo-de-mundo-cenas-e-entidades-compostas) | **Contratado** |
| API ~60 funções; adicionar exige remover/justificar; sem sinônimos | [§2.2](../../ENGINE_VISION2.md#22-minimalismo), [R16](../../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudança) | **Contratado** |
| Módulos opcionais (juice/phys/snd) não poluem a API básica nem o L0 | [§3.5](../../ENGINE_VISION2.md#35-plugins-tudo-que-quiser-carregar), [§5](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli) | **Contratado** |
| Jogo de referência na fase 3, antes de plugins/editores amplos | [§9](../../ENGINE_VISION2.md#9-roadmap), [§10](../../ENGINE_VISION2.md#10-backlog-e-organização-gratuito-minimalista) | **Contratado** |
| `scene.load`/`msg.send`/`evt.emit`/`attach`/`detach` (assinaturas finais após validar) | [§5 `[p]`](../../ENGINE_VISION2.md#5-referência-rápida-de-comandos-cli) | **Hipótese** |
| Ponte Rust↔Luau por buffers SoA | [§3.6-A](../../ENGINE_VISION2.md#36-performance-extremamente-veloz) | **Contratado (fase 6); alternativa a provar → ADR 0004** |
| Matemática: `libm` vs ponto fixo | [§3.6-C](../../ENGINE_VISION2.md#36-performance-extremamente-veloz) | **Hipótese → ADR 0004** |
| `luau-jit`/`@native` por função quente | [§3.6-B](../../ENGINE_VISION2.md#36-performance-extremamente-veloz) | **Hipótese → ADR 0004** |
| UI do editor: egui vs dogfooding em Luau | [§6.4](../../ENGINE_VISION2.md#64-implementação) *(decisão em aberto)* | **Hipótese → ADR 0004** |
| Múltiplos controladores; empilhamento de cenas; ossos; plugins nativos dinâmicos; networking; GPU raster; editor de cenas; tilemaps/diálogos no editor | [Fora do MVP — AGENTS §10](../../AGENTS.md#10-escopo-e-fases-9-11-13) | **Adiado** |
