# Núcleo executável (fase 1)

Fase 1 entregue (épico ENG-E01). Projeto mínimo = `engine.toml` + `main.luau` (+ `assets/` e `scenes/*.toml` opcionais).

## Comandos

```bash
engine check                # valida config, cena, script e assets; falta de arte = aviso
engine run --frames N --seed S      # roda headless em silêncio (ok = exit 0)
engine screenshot --out out/x.png   # run + PNG do frame final
# globais: --format toon|json
```

- Headless sempre (janela é fase 2). Erros saem em TOON (`code,file,line,msg,fix`), exit 1.
- `check: ok` com avisos `missing_assets[N]` quando sprite vira placeholder (caixa + rótulo — nunca erro silencioso).

## Contratos implementados

| Contrato | Comportamento |
|---|---|
| Tick (ordem fixa, §3.2) | barreira → `update(dt)` → `draw()` → raster → hash. `load()` conclui a montagem antes do 1º tick |
| Barreira (R8) | `ent.new` reserva ID e ativa na barreira; montar/matar pendente na mesma fila é válido |
| Handle geracional (R10) | índice + geração; slot reutilizado nunca revalida handle antigo (`E010`) |
| Dependência de componente | `Sprite` exige `Transform` (`E013`), validada em carga e runtime |
| Cena TOML (`schema = 1`) | `[[entities]]` com `key` única, tags, `transform`, `sprite`; erro de carga = `E031/E014` |
| R9 (draw não muta) | `ent.new/ent.kill` em `draw()` → `E016` |
| Orçamento de script (§4.9) | VM Luau sandbox (sem io/os), interrupção com orçamento por frame (`E050`) |
| R12 (TOON via encoder) | Toda saída passa por `Formatter` (`fmt-toon` padrão, `fmt-json` p/ `--format json`) |
| Determinismo (R7) | `rnd` = xoshiro256** com seed; hash FNV-1a do framebuffer; mesma seed = mesmos bytes |

## API Luau (fase 1)

```lua
gfx.clear(c) gfx.rect(x,y,w,h,c) gfx.rectb(x,y,w,h,c) gfx.spr(nome,x,y,[fx],[fy]) gfx.text(s,x,y,c) gfx.cam(x,y)
ent.new(tag,[x],[y]) -> e    ent.each(tag) -> {e,...}    ent.kill(e)
rnd(n)
```

## Testar

```bash
cargo test --workspace        # contratos do mundo + determinismo (hash fixo)
cargo xtask arch-check        # R1 (≤300 linhas/arquivo)
cargo xtask token-budget
```

Exemplo: `examples/hello` (cena + entidades + draw). Contratos do mundo: `crates/ecs/tests/world.rs`. Determinismo: `crates/cli/tests/determinism.rs`.
