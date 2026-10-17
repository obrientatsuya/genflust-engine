# O que foi feito e como funciona (fase 1)

Registro do épico ENG-E01 "Núcleo executável": o que a engine faz hoje, como foi construída e quais os limites desta fase.

## O que a engine faz hoje

Um projeto é uma pasta com `engine.toml`, `main.luau`, `assets/` e `scenes/*.toml` (opcionais). A engine:

1. **Monta** a cena TOML (entidades com `key`, tags, `transform`, `sprite`) antes do primeiro tick.
2. **Roda** `load()` → `update(dt)` → `draw()` em Luau (sandbox, `--!strict`), a 60Hz fixo, sem janela.
3. **Rasteriza** os comandos de desenho em CPU: framebuffer 320×180 indexado (paleta de 32 cores), fonte bitmap 5×7, sprites com flip, câmera.
4. **Exporta** PNG (`screenshot`) e um hash do framebuffer; asset ausente vira **placeholder** legível (caixa + rótulo).
5. **Valida** (`check`) config, cena, script e assets; todo erro sai estruturado em TOON (`code,file,line,msg,fix`).

## Como foi feita (crates e responsabilidades)

| Crate | Faz |
|---|---|
| `api` | Só contratos: `Command`, `Formatter`, `Value`, `EngineError`, constantes (320×180, 32 cores, tick 1/60) |
| `ecs` | Entidades com ID geracional (índice+geração), ciclo de vida com barreira, `Transform`/`Sprite`, nomes/tags |
| `core` | Relógio 60Hz, RNG xoshiro256** (seed), cena TOML (`schema = 1`), orquestração do tick |
| `gfx` | Framebuffer indexado, paleta (PICO-8 padrão, sobrescrevível por arquivo), raster, fonte, PNG (encoder), hash FNV-1a |
| `assets` | PNG → sprite indexado (cor mais próxima da paleta), registro por nome, placeholders determinísticos |
| `script` | VM Luau (sandbox, sem io/os), bindings `gfx.*`/`ent.*`/`rnd`, orçamento por interrupção, erros contidos |
| `fmt-toon` / `fmt-json` | Encoders de saída (`Formatter`); TOON é o padrão |
| `cli` | Comandos `check`/`run`/`screenshot`, flags, exit codes, saída de erro |
| `app` | Composition root: único lugar que junta as impls concretas (binário `engine`) |

Direção de dependência: ferramentas → capacidades → núcleo → `api`. O `core` não conhece Luau nem gfx ([R4](../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudanca)): o `app` injeta os passos do tick.

## Como um frame acontece

```
barreira (aplica criações/remoções pendentes)
  → update(dt) [Luau + rnd]
  → draw() [só emite comandos; ent.new/kill bloqueado aqui — R9](../AGENTS.md#1-regras-duras-violou--rejeitar-a-mudanca)
  → raster (comandos → framebuffer)
  → hash (FNV-1a) e, se pedido, PNG
```

`load()` conclui a montagem antes do 1º tick. `ent.new` reserva o ID no tick e ativa na barreira; matar uma pendente na mesma fila cancela a criação; slot reutilizado nunca revalida handle antigo (E010).

## Como o determinismo funciona

Timestep fixo (1/60), RNG semeado (`--seed`), sem relógio de parede, ordem de iteração estável. O teste compara hashes: mesma seed → mesmos bytes; seeds diferentes → bytes diferentes — em qualquer SO.

## Limites desta fase (ainda não existe)

| Adiado | Fase |
|---|---|
| Janela, input, `state`/`log`/`filmstrip`, replay | 2 |
| Controladores por entidade, `cycle`, `test --expect`, attach/detach, troca de cena | 3 |
| `why`, snapshots, golden hashes nos 2 SOs, plugin host | 4 |
| Docs geradas, `--budget`, MCP, Live View | 5 |
| `juice`, áudio, kits, ponte por buffers, `perf-budget` com baseline | 6 |
