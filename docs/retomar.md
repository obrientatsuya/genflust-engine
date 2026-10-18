# Retomar o trabalho (qualquer pessoa ou LLM)

Ponto de entrada para continuar o desenvolvimento **sem alucinar**. Leia nesta ordem e **verifique a realidade antes de confiar em memória**.

## 1. Regras e fonte de verdade

- `AGENTS.md` — regras duras (R1–R21), arquitetura, o fluxo de "antes de concluir".
- `ENGINE_VISION2.md` (§N) — fonte de verdade do produto; em conflito, **o doc vence**.
- Não inventar API, flag, comando ou formato fora do doc; itens `[p]`/hipótese não são contrato.

## 2. Onde está o trabalho

- **Kanban:** https://github.com/users/obrientatsuya/projects/2 — a task em *In Progress* é onde parou.
- **Issues:** cada task tem critério de aceite; os comentários de **aceite** registram evidência do que foi feito.
- **Backlog completo:** `ENGINE_BACKLOG.toon` (92 itens; dá pra buscar com `cargo xtask search`).

## 3. Verificar a realidade (anti-alucinação)

```bash
cargo test --workspace        # contratos do mundo + determinismo
cargo xtask arch-check        # R1: nenhum arquivo >300 linhas
cargo xtask token-budget      # llms.txt ≤ 3k tokens
```

CI verde no `main` (ubuntu+windows) = estado bom. Se um teste falha, o problema é real — não presuma.

## 4. Buscar sem ler tudo

```bash
cargo xtask search "tick ordem"        # trechos certos, com arquivo:linha, em TOON
```

Camadas de contexto: `llms.txt` (L0, a API inteira) → `docs/adr` (decisões) →
`docs/nucleo-*.md` (fase 1) → `docs/ambiente.md` (setup) → este arquivo.

## 5. Fluxo de trabalho (como nas tasks anteriores)

1. Pegue a task *In Progress* no kanban (ou mova a próxima de *Todo*).
2. Leia a issue + a `§` citada; menor mudança que cumpre o aceite.
3. `cargo test` + `cargo xtask arch-check` (+ budgets quando aplicável).
4. Commitamos, registramos o **aceite na issue** e movemos o kanban.
5. Só commitar as mudanças da task — arquivos pessoais do dono ficam fora do git (untracked).

## 6. Estado atual (18/10/2026)

- Fase 1 **aceita** (issue #1): núcleo executável, renderer software, determinismo. CI verde.
- ENG-01-01 **aceita** (issue #2): ADRs com contratos × hipóteses em `docs/adr/`.
- Próximas: ENG-01-02 (provar dependências com protótipos) e o restante da fase 1 — ordem no kanban.
- Fase 2 (janela/input) só depois; editores só nas fases 7–9.
