# Ambiente (Windows e Linux)

O repo compila e roda nos dois sistemas; o CI é o árbitro (PR só entra com os dois verdes). Setup completo abaixo. Pré-requisito comum: `gh auth login`.

## Setup — Windows

```powershell
winget install Git.Git Microsoft.PowerShell Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --quiet --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
git config --global core.autocrlf false   # o .gitattributes manda o LF
```

- Use `pwsh` (PowerShell 7). O 5.1 grava `>` em UTF-16 e corrompe PNG/golden; a engine grava por `--out`, nunca por redirecionamento.
- MSVC + Build Tools (C++) é obrigatório: o mlua compila o Luau em C++.
- WSL2 (Ubuntu) serve só para reproduzir falhas do Linux; clone dentro do WSL (`~/`), não em `/mnt/c`.

## Setup — Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install build-essential pkg-config   # mold é opcional
```

- X11/Wayland/ALSA só entram com as features `window`/`audio`; headless não precisa de nada disso.

## Clonar e verificar

```bash
gh repo clone obrientatsuya/genflust-engine
cd genflust-engine
cargo xtask arch-check        # rustup instala o 1.99.1 do rust-toolchain.toml sozinho
```

Rodar localmente o mesmo que o CI (ubuntu + windows):

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                     # inclui golden hashes
cargo xtask arch-check
cargo xtask token-budget
```

## Por quê (curto)

- **LF em tudo** (`* text=auto eol=lf`): CRLF muda os bytes do arquivo → quebra o hash dos goldens e o diff dos TOML.
- **`lower_snake_case` em asset/cena**: Windows ignora caixa, Linux não; dois nomes que diferem só por caixa colidem em um e não no outro. O `check` rejeita.
- **Caminhos sempre com `/`**: separador válido nos dois SOs.
- **Float via `libm`, sem fast-math, sem `-C target-cpu=native`**: mesmo resultado bit a bit nos dois (R7); golden hashes commitados batem nos dois.
- **`Cargo.lock` versionado + toolchain fixa**: build reproduzível, mesmos warnings nos dois.
- **mold/lld só em linux-gnu**: nunca afeta o Windows; opcional.
- **Hot reload/editor: temporário + rename, debounce no watcher**: no Windows, arquivo aberto por outro processo trava a escrita.
- **`perf-budget` fora do CI compartilhado**: runner compartilhado varia mais que a meta de 5%; roda manual em máquina self-hosted com baseline em `perf/baselines.toml`.
- **IA local (RTK, skills em `.agents/`) fora do git**: quem não usa IA não é afetado.
