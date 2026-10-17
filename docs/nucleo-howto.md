# Rodar e verificar (fase 1)

Pré-requisito: toolchain e CI local em `docs/ambiente.md`.

## 1. Instalar o binário (uma vez)

```bash
cargo install --path crates/app    # deixa `engine` no PATH (~/.cargo/bin)
```

Sem instalar, equivale a: `cargo run --bin engine -- <comando>`.

## 2. Rodar dentro da pasta do projeto

O comando funciona onde existe `engine.toml` (ex.: uma pasta de jogo, ou `examples/hello`).

```bash
cd examples/hello

engine check                                  # valida config, cena, script e assets
engine run --frames 60 --seed 42              # roda headless; silêncio = sucesso
engine screenshot --frames 60 --seed 42 --out out/demo.png
```

Saídas esperadas:

- `check` → `check: ok` (exit 0). Placeholders aparecem como aviso, nunca erro:
  ```
  missing_assets[2]{name}:
    coin
    player_idle
  ```
- `run` → nada (exit 0). Falha = tabela `errors[N]{code,file,line,msg,fix}` (exit 1).
- `screenshot` → PNG 320×180 em `out/` (criado se faltar).

## 3. Verificar a entrega (na raiz do repositório)

```bash
cargo test --workspace        # contratos do mundo + determinismo
cargo xtask arch-check        # R1: nenhum arquivo >300 linhas
cargo xtask token-budget
```

O CI roda exatamente isso em ubuntu-latest + windows-latest, mais `fmt --check` e `clippy -D warnings`.

## Erros comuns

| Código | Causa | Correção |
|---|---|---|
| `E020` | rodou fora da pasta do projeto (sem `engine.toml`) | `cd` para a pasta do jogo |
| `E050` | erro no `main.luau` (sintaxe ou runtime) | a mensagem aponta a linha; corrija e rode de novo |
| `E010` | handle obsoleto (`ent.kill`/consulta após remoção) | use o handle do ciclo de vida atual |
| `E016` | `ent.new`/`ent.kill` chamado em `draw()` | mova para `update()`/`load()` (R9) |
| `E103` | não conseguiu gravar o PNG | verifique permissões do diretório de `--out` |

Onde ficam os testes: `crates/ecs/tests/world.rs` (contratos do mundo) e `crates/cli/tests/determinism.rs` (mesma seed → mesmo hash).
