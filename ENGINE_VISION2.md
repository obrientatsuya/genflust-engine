# Genflust Engine

> Rust core + Luau scripting. Pixel art/2D. Open source, full modular, CLI-first, extremely agent friendly (FIRST).

> **Status:** visão de produto e arquitetura em planejamento. Revisão integrada em 16/10/2026. APIs e exemplos novos são propostas a validar, não funcionalidades já implementadas.

---

## 0. TL;DR

| Pilar | Decisão |
|---|---|
| Eficiência | Renderer por software em framebuffer low-res, command buffer, FFI em batch, binário <5MB |
| Minimalismo | API de ~60 funções, resolução e paleta fixas, projeto = pasta pequena |
| Clareza visual | Pixel-perfect, paleta limitada, debug overlay, filmstrip |
| CLI | Toda funcionalidade é um comando. GUI e MCP são wrappers |
| Luau + Rust | Rust executa, Luau decide. Luau pelo typecheck estático e sandbox |
| Arte | **Humano desenha.** Editor de pixel art e VFX embutido. Agente usa por nome, com placeholders |
| Modularidade | Núcleo pequeno, contratos SOLID e capacidades opcionais. Plugins Rust estáticos + Luau em runtime |
| Modelo de mundo | Cenas contêm entidades compostas por dados e controladores; ECS/SoA em Rust, sem hierarquia de classes |
| Juiciness | **Live View:** ver o agente trabalhando em tempo real, claro e satisfatório |
| Formato de saída | **TOON** por padrão (30-60% menos tokens que JSON em dados uniformes). JSON opcional |
| Performance | **Rápido por design:** Luau declara, Rust simula; ECS/SoA, zero alocação; raster indexado + SIMD; medido no CI |
| Economia de contexto | Silêncio no sucesso, delta, `--budget` por saída, `cycle`, `map`, `resume` |
| **Agent-friendly** | Observar, verificar, corrigir sem adivinhar. Contexto mínimo. API conhecida |

---

### 0.1 Direção da revisão

A visão agent-first deste documento continua sendo a referência. A contribuição complementar foi incorporada onde reduz código repetido, torna o comportamento verificável ou evita custo obrigatório no runtime: cenas, composição de entidades, controladores reutilizáveis, comunicação explícita e associação entre objetos.

Essas ideias entram sobre o ECS/SoA e o CLI existentes. Os tipos de ator viram presets de dados; a criação visual usa os mesmos arquivos e contratos do CLI. O MVP prioriza um jogo pequeno completo antes de ampliar editores e extensões.

A seção 13 registra a análise da integração: ideias aproveitadas, alternativas adaptadas ou retiradas e funcionalidades adiadas, com o motivo de cada decisão.

---

## 1. Análise: tensões e como resolvê-las

**1. Agente não conhece a engine (sem treinamento).**
→ Imitar a forma de APIs que ele já conhece (LÖVE `load/update/draw`, PICO-8). Tudo que for novo é ensinado sob demanda via CLI.

**2. Agent-friendly pede muitas ferramentas; minimalismo pede poucas.**
→ Uma única superfície: o CLI. Poucos comandos, saída previsível, composáveis. MCP é só um wrapper fino.

**3. Eficiência vs. observabilidade.**
→ Renderer por software deterministico: barato em 320×180, roda sem GPU, screenshots idênticos em qualquer SO.

**4. Arte é do humano; o agente é bom em texto.**
→ O agente programa e pede arte; o usuário desenha em um editor embutido (sem trocar de programa). Contrato por nome + placeholders mantêm o jogo testável sem arte pronta. Metadados, VFX e cenas são texto, editáveis pelos dois.

**5. Agente enche o contexto.**
→ Docs em camadas (L0/L1/L2), saídas truncadas por padrão, erros estruturados com a correção embutida.

**6. Humano precisa acompanhar o agente.**
→ Live View (7.2): cada ação do agente vira feedback visual e sonoro claro, em tempo real, assinando o barramento de eventos.

**7. Tudo precisa ser extensível sem inchar o core.**
→ Contratos pequenos em `api`, capacidades opcionais injetadas no `app` e dependências declaradas. O núcleo mantém identidade, ciclo de vida e ordem de execução; física, áudio e efeitos são módulos.

**8. Cada ida e volta do agente custa tempo e tokens.**
→ `engine cycle` em 1 chamada, silêncio no sucesso, deltas e orçamento de tokens por saída (4.13, 4.14).

**9. Velocidade extrema vs. simplicidade para o agente e determinismo.**
→ As otimizações vivem dentro do Rust; a API Luau continua simples (`ent.pos(e)`), buffers e SIMD são invisíveis. Matemática determinística (`libm`/ponto fixo) e golden hashes entre sistemas operacionais (3.6).

**10. Composição livre vs. previsibilidade para o agente.**
→ Uma única identidade de entidade, componentes opcionais, controladores reutilizáveis e relações explícitas. Ordem de atualização, mensagens e remoção têm regras fixas (3.7–3.10).

**Princípio guia:** *cada feature precisa responder: o agente consegue usar, verificar e consertar isso só com texto e poucos tokens?* Se não, não entra.

---

## 2. Os seis pilares

### 2.1 Eficiência
- Update fixo a 60Hz, render desacoplado.
- Zero alocação no hot path. Command buffer pré-alocado, esvaziado 1x por frame.
- Nunca 1 chamada FFI por entidade. Sempre batch.
- Handles inteiros e buffers na fronteira crítica Rust/Luau. Nomes de assets, cenas e componentes são resolvidos em carga/check; mensagens frequentes usam schemas e filas pré-alocadas.
- Meta: binário <5MB, boot <100ms, idle <30MB RAM, headless >10k frames/s. Detalhes e orçamentos em 3.6.
- Cargo: `lto="fat"`, `codegen-units=1`, `opt-level=3`, `panic="abort"`, `strip=true`.

### 2.2 Minimalismo
- Resolução interna fixa (padrão 320×180), escala inteira.
- Paleta fixa (padrão 32 cores, trocável por arquivo de texto).
- API pública pequena (~60 funções básicas). Cenas, relações e comunicação entram nesse orçamento; kits e capacidades opcionais documentam apenas sua API ativa. Adicionar função exige remover ou justificar.
- Projeto mínimo = `main.luau`, `assets/`, `engine.toml`, `AGENTS.md`. `scenes/`, `entities/` e `controllers/` aparecem conforme a necessidade; uma cena implícita permite começar em um único script.
- Editores embutidos, enxutos e opcionais (feature `editor`), todos falando PNG + texto simples.

### 2.3 Clareza visual
- Pixel-perfect: sem filtro, sem subpixel, escala inteira.
- Paleta limitada força legibilidade.
- Debug overlay: hitboxes, IDs, grid, FPS, estado de entidades selecionadas.
- Tudo que o agente "vê" tem versão para humano (screenshot, filmstrip, viewer).

### 2.4 CLI-first
- Todo comando funciona sem janela.
- Saída estruturada em **TOON** por padrão (ver 4.12), `--format json` para interoperabilidade, `--full` para detalhes.
- Exit codes significativos. Idempotente quando possível.
- GUI/viewer e MCP chamam o mesmo core que o CLI.

### 2.5 Luau + Rust
**Regra de ouro: Luau decide, Rust executa.**

| Rust | Luau |
|---|---|
| Loop, cenas, identidade/ciclo de vida, janela, input, áudio | Regras do jogo |
| Renderer, sprite batch, tilemap, paleta | Controladores e comportamento de entidades |
| Colisão (spatial hash), partículas, pathfinding | State machines, decisões de IA |
| Tween/easing core, câmera, shake | UI, diálogos, level scripts |
| Assets, atlas, serialização | Combinação de módulos |
| CLI, headless, replay, hot reload, MCP | Testes do jogo |

**Por que Luau e não LuaJIT:** tipos opcionais, análise estática (`engine check` acusa erro antes de rodar), sandbox nativo, boa integração via `mlua`. LuaJIT é mais rápido, mas congelado em 5.1 e sem typecheck. Em 2D leve com loops pesados em Rust, a diferença não importa.

### 2.6 Agent-friendly
Ver seção 4. É o pilar central.

---

## 3. Arquitetura

### 3.1 Renderer por software (decisão-chave)
Em 320×180, rasterizar em CPU é barato e dá:
- Zero dependência de GPU, roda em CI/servidor.
- Screenshots **bit-idênticos** entre Linux e Windows (testes de regressão confiáveis).
- Janela só faz upload do framebuffer como textura (`softbuffer` ou `miniquad`).
- Menos código, menos bugs, binário menor.

### 3.2 Fluxo de um frame
```
[início do tick] -> aplicar mudanças estruturais/transição pendentes
                -> entregar mensagens e eventos de gameplay do tick anterior
input -> [Luau update(dt) + controladores ativos] -> comandos de gameplay
      -> [Rust] systems habilitados + relações/transformações (batch)
      -> eventos ordenados + estado observável do tick
      -> [Luau draw()] -> command buffer
      -> [Rust] raster -> framebuffer -> janela | PNG | hash
```
`draw()` apenas produz comandos visuais; não altera a simulação. Criar/remover entidades, trocar componentes/controladores ou mudar de cena durante um tick agenda a alteração para o início do próximo. Em `load()`, a montagem inicial é concluída antes do primeiro tick. As fases e a ordem dos systems são estáveis e fazem parte do contrato de replay.

### 3.3 Workspace (Cargo)
Regra para código-fonte: **nenhum arquivo passa de 300 linhas** (alvo 200). Um propósito por módulo, funções pequenas. A árvore abaixo descreve responsabilidades previstas, não uma obrigação de criar todos os crates no MVP. Começar com módulos internos e extrair um crate quando houver fronteira de dependência, feature opcional ou reuso real.

```
crates/
  api/          # SÓ traits e tipos: Renderer, InputSource, AudioSink,
                #   AssetLoader, Command, Panel, Fx, EventSink, Formatter. Zero implementação
  events/       # barramento de eventos tipado (base de Live View, log, replay, MCP)
  core/         # loop, tempo, RNG, cenas leves, ciclo de vida e fases de execução
  gfx/          # impl Renderer (software, indexado): paleta, sprites, tilemap, texto
  input/        # impl InputSource, gravação/replay
  audio/        # impl AudioSink
  ecs/          # entidades/componentes SoA, IDs com geração, relações, buffers Luau
  systems/      # systems opcionais por dados: movimento, gravidade, animação, vida útil
  physics/      # AABB, spatial hash, raycast
  juice/        # tween, shake, hitstop, partículas, squash
  script/       # mlua/Luau, sandbox, bindings
  assets/       # loaders (PNG+toml), atlas, placeholders
  plugins/      # plugin host: manifest, carga, permissões, registry
  cli/          # registry de comandos, roteia a saída via Formatter
  fmt-toon/     # impl Formatter padrão (crate `toon-format`, spec v3.0)
  fmt-json/     # impl Formatter p/ interoperabilidade
  agent/        # filmstrip, ascii, state dump, diff, playtest, cycle, why,
                #   snapshot/rewind, map, status, resume
  profiler/     # timers por sistema, contadores de FFI/alocações, bench
  live/         # Agent Live View: painéis, efeitos, temas
  window/       # viewer de janela
  editor-core/  # shell do editor (feature "editor")
  editor-pixel/ # editor de pixel art
  editor-vfx/   # editor de VFX e curvas
  editor-scene/ # futuro: cena + inspetor, mesma serialização e validação do CLI
  mcp/          # wrapper fino sobre o CLI
  app/          # composition root: único lugar que conhece impls concretas
kits/           # plugins Luau oficiais (platformer, topdown, shmup), fora do core
xtask/          # arch-check: linhas, grafo, contratos | token-budget: orçamentos de saída | perf-budget: fps e ms por sistema
```
Feature flags previstas: `window`, `audio`, `physics`, `animation`, `tilemap`, `juice`, `mcp`, `live`, `editor`. Runtime mínimo de referência: `api + events + core + ecs + gfx + input + script + assets + cli + fmt-toon + app`; sem janela no modo headless. São responsabilidades lógicas, que podem compartilhar crate inicialmente.

**Três camadas:** núcleo obrigatório (tempo, identidade, cenas, ciclo de vida e contratos), módulos de capacidade (render, áudio, física etc.) e ferramentas (CLI, MCP, Live View, editores). O runtime de referência inclui o renderer software; áudio, física e partículas só entram quando usados. `engine.toml` declara as capacidades do projeto; `check`/`build` validam se estão disponíveis na distribuição da engine.

### 3.4 SOLID com fronteiras concretas

| Princípio | Regra na engine | Verificação |
|---|---|---|
| **S** | Um módulo, uma razão para mudar. ≤300 linhas por arquivo | `xtask arch-check` falha o CI |
| **O** | Estender uma capacidade = adicionar plugin/impl. Alterar contratos do núcleo exige justificativa e versão | Pontos de extensão declarados usam registry |
| **L** | Qualquer impl de uma trait substitui outra sem o core notar (ex.: renderer software ↔ GPU) | Suíte de testes de contrato por trait, rodada em todas as impls |
| **I** | Traits pequenas, uma capacidade cada, ≤5 métodos. Nada de trait "Deus" | `arch-check` conta métodos |
| **D** | Coordenação depende de contratos e tipos; implementações de capacidades são injetadas no `app` | Grafo sem ciclos; núcleo não importa backends ou ferramentas |

Regras complementares:
- Sem globals nem singletons. Dependências entram por construtor.
- Consultas e comandos diretos passam por contratos públicos; notificações usam eventos (`events`). Evitar transformar uma consulta simples em uma cadeia de eventos. O núcleo pode usar módulos internos de dados, como ECS, sem depender de backends concretos.
- Ciclos de dependência proibidos. Traits marcam fronteiras reais de substituição; funções e estruturas internas não precisam virar plugins.
- Cada crate é testável isolado com fakes das traits.
- As regras valem para quem contribui (humano ou agente): ficam no `AGENTS.md` do repositório da engine e são verificadas por `xtask arch-check`.

### 3.5 Plugins: tudo que quiser carregar

**Pontos de extensão:** backend de render, áudio, input, formato de asset, formato de saída (`Formatter`), comando de CLI, tool de MCP, ferramenta de editor, painel/tema do Live View, efeito de VFX, módulo de gameplay, validador do `check`, gerador de docs.

| Nível | Como | Para quê |
|---|---|---|
| Estático (Rust) | Crate + feature flag, registra a impl no `app` | Módulos oficiais e pesados, custo zero |
| Dinâmico (Luau) | `plugins/<nome>/` com `plugin.toml`, carregado em runtime, sandbox + permissões | Gameplay, comandos, painéis, efeitos, validadores |
| Futuro (WASM) | Módulo isolado | Plugins nativos de terceiros (evitar dylib Rust: ABI frágil) |

```toml
# plugins/platformer/plugin.toml
[plugin]
name = "platformer"
version = "0.1.0"
api = "~0.1.0"              # faixa compatível do contrato da engine
requires = ["physics"]

[permissions]
fs = "project"              # none | project

[docs]
l0 = "platformer.move(e,dx)  anda com gravidade e colisão"
```

**Plugins também são agent-friendly:**
- Trazem `l0`, `docs/`, `examples/` e `.d.luau`. Entram nas camadas L0/L1/L2 automaticamente.
- O L0 (`llms.txt`) só lista plugins **ativos**: o contexto cresce apenas com o que se usa.
- `engine plugin add|rm|list|info`. Instalação por pasta ou URL git.
- Erro de plugin sempre diz o nome e a capacidade afetada. Falha em ferramenta opcional desativa a ferramenta; falha em gameplay ou dependência obrigatória pausa a simulação com diagnóstico. O processo permanece disponível para inspeção.
- API versionada e faixa compatível explícita. Antes de 1.0, quebra de contrato exige nova versão minor e migração; a partir de 1.0, major. Fixar revisões/versões instaladas para reproduzir o projeto.
- Manifest declara dependências e capacidades fornecidas. Resolver e validar o grafo antes de executar: versão incompatível, dependência ausente ou ciclo geram erro com correção. Um componente de áudio/física não pode ativar silenciosamente um módulo indisponível.
- Começar com módulos Rust estáticos e scripts Luau. Plugins nativos dinâmicos ficam fora do MVP; novas abstrações de extensão precisam provar uso no jogo de referência.

### 3.6 Performance: extremamente veloz

Princípio: **rápido por design, provado por medição.** Há um orçamento de frame, o hot path não aloca, e só se otimiza o que o `engine profile` apontar.

**Metas (a validar com benchmark):**

| Métrica | Meta |
|---|---|
| Boot | <100ms |
| Frame de jogo típico (~1000 entidades) | <2ms de CPU (orçamento total: 16,6ms a 60Hz) |
| 10k entidades em systems Rust | <2ms |
| Chamadas FFI Rust↔Luau por frame | <200 |
| Alocações no hot path | 0 |
| Headless | >10k fps (jogo simples) |
| Input → tela | ≤1 frame |
| Pausa de GC do Luau | <0,5ms por frame |
| Memória idle / binário | <30MB / <5MB |

**A. Fronteira Rust ↔ Luau (maior custo)**
- **Luau declara, Rust simula.** Systems embutidos em Rust (movimento, gravidade, atrito, animação, vida útil, colisão) são configurados por dados. Luau roda só decisões e eventos. Mais rápido e menos código para o agente escrever.
- **Ponte por buffers:** o estado das entidades fica em Rust num buffer contíguo (SoA) que o Luau lê pelo tipo nativo `buffer`, sem FFI por campo. Comandos de desenho/ação voltam por outro buffer. Posições usam o tipo nativo `vector` (sem alocação, sem GC).
- **O agente nunca vê buffers:** a API continua simples (`ent.pos(e)`, `ent.set_vel(e, vx, vy)`) e lê/escreve nos buffers por baixo.
- Handles inteiros, sem tabela por entidade.

**B. Luau rápido**
- A geração de código nativo do Luau (x64/ARM64) costuma dar 1,5x a 2,5x em código intensivo de cálculo. No `mlua` ela entra pela feature `luau-jit`, ainda experimental. Marcar só as funções quentes com `@native`/`--!native`, **medir antes de ligar por padrão**, e manter o interpretador como fallback (que já é rápido).
- `--!strict` em todo o código: ajuda o `check` e o código nativo.
- **GC com orçamento:** passo incremental no fim do frame.
- Pools e zero closures/tabelas criadas por frame (lint do `check`).
- Bytecode pré-compilado no `engine build`; hot reload só em dev.

**C. Dados e simulação (Rust)**
- **ECS/SoA:** arrays contíguos, IDs com geração (`u32` índice + geração), sem `Box`/`Rc` no hot path.
- **Zero alocação por frame:** bump arena resetada a cada frame, `smallvec` para listas pequenas.
- **Colisão:** spatial hash com células fixas, broadphase incremental.
- **Matemática determinística entre SOs:** não usar o `libm` do sistema (`sin`/`cos` podem diferir entre Linux e Windows e quebrar replays). Usar a crate `libm` (Rust puro) ou ponto fixo na física; sem fast-math. Golden hashes no CI nos dois SOs.

**D. Renderer por software**
- **Framebuffer indexado (u8):** 320×180 = ~57KB, cabe no cache. A paleta vira RGBA só no final, por LUT com SIMD.
- Blit por linha (`memcpy` para sprite opaco), máscara em vez de branch por pixel, clipping antes de rasterizar.
- Ordenação por camada com counting sort inteiro, atlas único (batch contíguo).
- **GPU só como apresentador:** um quad com textura 320×180 e `nearest`. Escala inteira, vsync e baixa latência de graça. A rasterização continua na CPU, preservando o determinismo.
- Multithread só se medir: tiles disjuntos continuam determinísticos. Padrão = single-thread. Pipeline (update N+1 enquanto renderiza N) é opção.

**E. Latência e pacing**
- Timestep fixo com interpolação no render, spin-sleep híbrido.
- Input lido no início do frame e apresentado logo após o update.

**F. Boot, assets e áudio**
- **`.pak` derivado no build:** atlas + metadados binários com mmap e zero parse. A fonte da verdade continua sendo PNG + TOML; o `.pak` é só artefato.
- Carga preguiçosa de assets.
- **Áudio:** thread própria, mixer sem alocação, ring buffer lock-free, samples pré-decodificados.

**G. Build**
- PGO além de LTO fat, `codegen-units=1`, `panic=abort`.
- Duas variantes de SIMD (baseline e AVX2) com detecção em runtime.
- Linker `mold`/`lld` em dev. Plugins Luau não exigem recompilar a engine.

**H. Medição**
- `engine profile`: ms por sistema, FFI e alocações, em TOON curto (o agente otimiza sozinho com isso):
```
frame_ms: 1.4
systems[4]{name,ms,ffi,allocs}:
  luau.update,0.62,38,0
  physics,0.31,0,0
  raster,0.38,0,0
  audio,0.05,0,0
```
- `engine bench`: headless determinístico, mesma seed, mesmo resultado.
- `xtask perf-budget` no CI: falha se o fps headless cair mais de 5% ou algum sistema passar do orçamento.
- Testar também em hardware fraco (CPU antiga, Steam Deck).

### 3.7 Modelo de mundo: cenas e entidades compostas

**Modelo mental:** `Cena → Entidades → Componentes + Controlador`. **Implementação:** ECS/SoA em Rust. “Ator” é a descrição conceitual de uma entidade; a API pública continua `ent`, sem uma segunda família `actor`.

- **Cena:** conjunto de entidades, nomes/tags locais, dados e contexto de execução. O MVP tem uma cena ativa; empilhamento e streaming ficam para depois. Uma cena implícita atende projetos só com `main.luau`.
- **Entidade:** ID opaco com índice + geração, cena de origem, estado de ciclo de vida, nome opcional e tags. Reutilizar um slot não torna válido um handle antigo. IDs de runtime não são referências permanentes em arquivos; cenas usam chaves locais, resolvidas em carga.
- **Componentes:** dados opcionais (`Transform`, `Sprite`, `Collider`, `AudioSource`, `Animator` etc.). Uma entidade pode ser apenas lógica, sem posição, sprite ou colisão. Dependências são explícitas: `Sprite` exige `Transform`; `Collider` exige `Transform` e a capacidade de física.
- **Controlador:** comportamento que usa os dados e as APIs públicas. Inventário, equipamento e diálogo pertencem aos kits/plugins de gameplay; não são campos obrigatórios de toda entidade.
- **Ciclo de vida:** criação pendente → ativa → remoção pendente → removida. Uma remoção solicitada marca a entidade e impede novos comandos de gameplay para ela; a limpeza acontece na próxima barreira estrutural, sem invalidar uma iteração em curso. `ent.new` pode reservar um ID durante o tick; ele só fica ativo na barreira. Comandos de montagem podem usar esse ID pendente na mesma fila, com validação antes da ativação.
- **Persistência:** cenas são TOML, comportamento é Luau, pixels são PNG. Nenhum arquivo de cena exige serializar closures, ponteiros ou objetos de uma VM.

Exemplo conceitual de `scenes/arena.toml` (schema e nomes sujeitos ao orçamento da API):
```toml
schema = 1
name = "arena"

[[entities]]
key = "player"
tags = ["player", "character"]
[entities.components.transform]
x = 48
y = 120
[entities.components.sprite]
asset = "player_idle"
[entities.controller]
module = "controllers/human.luau"

[[entities]]
key = "enemy_spawn"
tags = ["spawn"]
[entities.components.transform]
x = 200
y = 120
```
`engine check` valida schema, chaves únicas, componentes, módulos, scripts e referências. Assets ainda não desenhados seguem a regra de placeholders (4.8), com diagnóstico explícito. CLI e editor usam o mesmo carregador e validador; o editor salva o mesmo TOML, com ordem estável para diff.

**Transição de cena:** preparar e validar a próxima cena antes de trocar; em caso de falha, manter a atual. A troca ocorre na barreira do tick e limpa entidades, ouvintes e filas da cena anterior. No MVP, dados entre cenas passam por estado de sessão explícito e serializável, sem carregar handles antigos. Mudanças de componentes e relações são validadas também em runtime, não apenas em `check`.

### 3.8 Controladores reutilizáveis e presets

Player e NPC compartilham componentes e sistemas de personagem. O que muda é a origem das intenções: input humano, replay ou IA. O controlador decide; sistemas Rust aplicam movimento, colisão e animação em batch. NPC não herda de Player nem depende de teclado.

**Contrato mínimo de controlador Luau:** callbacks opcionais `load(e)`, `update(e, dt)`, `on_message(e, msg)`, `on_event(e, evt)` e `unload(e)`. Recebem um handle validado e o contexto público, sem acesso ao armazenamento interno. `main.luau` mantém `load/update/draw`; depois de seu `update`, controladores ativos rodam por ID estável antes dos systems Rust. Mensagens/eventos são entregues na fase inicial descrita em 3.2.

- Um controlador por entidade no MVP. Ele pode compor funções Luau pequenas, como mover, atacar e interagir, em ordem explícita. Múltiplos controladores independentes só entram depois de definir prioridade, fases e conflito de escrita; não há “último a executar vence” implícito.
- `load` roda uma vez ao ativar; `unload`, uma vez na remoção ou substituição. Remover entidade/controlador cancela suas inscrições e recursos registrados; falhas no encerramento não impedem a limpeza feita pelo host.
- Estado persistente do controlador é separado do código, composto de dados serializáveis. Snapshot/replay inclui esse estado. Hot reload preserva dados compatíveis; mudança de schema exige migração declarada ou reinício explícito da cena.
- Não criar callback Luau por partícula/projétil só para integrar posição. Comportamento uniforme é system Rust ou operação em batch; callbacks ficam para decisões que precisem deles.

Os nomes de ator úteis da visão complementar viram **presets opcionais de componentes**, fornecidos pelos kits. Não são subclasses nem ampliam a API básica:

| Preset | Composição e utilidade |
|---|---|
| `player` / `npc` | Dados de personagem compartilhados + controlador humano ou IA |
| `static` | Transform e, conforme necessário, Sprite/Collider estático; aceita comportamento adicional |
| `speaker` | AudioSource, com Transform apenas quando usar posição ou associação espacial; áudio opcional |
| `empty` | Marcador com Transform ou entidade apenas lógica; sem Sprite obrigatório |
| `relay` | Controlador de condições/eventos: coletar chaves → mandar abrir uma porta |
| `spawner` | Configuração de spawn + controlador/sistema; substitui o nome provisório “ChunKy” por um nome descritivo |

Criar entidades continua sendo capacidade geral; não exige um `spawner`. Speaker é uma composição de AudioSource, não um segundo sistema de áudio. Efeitos cosméticos continuam no sistema de partículas; só viram entidades quando precisam de identidade ou interação de gameplay.

### 3.9 Comunicação: mensagens dirigidas e eventos

| Mecanismo | Intenção | Exemplo conceitual |
|---|---|---|
| Consulta pública | Ler estado, sem efeito colateral | `ent.pos(e)` |
| Mensagem dirigida | Solicitar ação de um destinatário | `msg.send(door, "open", {})` |
| Evento | Notificar um fato aos interessados da cena | `evt.emit("item_collected", {item="key"})` |

Novos nomes de API são propostas; schemas/tipos devem gerar `.d.luau`, docs e validação a partir da mesma definição. Mensagens e eventos de gameplay são enfileirados, nunca callbacks recursivos síncronos. Ações locais simples continuam usando funções públicas, sem passar obrigatoriamente pelo barramento.

- Entrega no início do próximo tick, em ordem estável de emissão e de assinantes. Cada item registra cena, tick, origem, destino quando houver, tipo e sequência. Eventos do SO/entrada são normalizados antes da simulação.
- Destinatário precisa ser válido e pertencer ao escopo permitido. Mensagem para entidade removida é descartada com diagnóstico estruturado; `test` pode tratar isso como falha. Uma referência obrigatória inválida na cena é erro de carga.
- Inscrições pertencem à entidade/controlador/cena e são removidas automaticamente. Payloads têm schema, tamanho limitado e representação serializável; referências a entidades são revalidadas na entrega.
- Filas pré-alocadas e limites de capacidade, entrega por tick e repetição de cadeias impedem tempestades de eventos. Exceder um limite produz erro reproduzível; eventos de gameplay não são descartados silenciosamente.
- `engine log`, `state` e `why` mostram origem, destino e cadeia causal sob demanda. As filas pendentes entram em snapshots e no hash de estado; replay exige a mesma versão de código/assets e configuração.

O barramento de observação publica também reload, check, testes e diagnósticos para o Live View. Seus consumidores não alteram a simulação nem recebem poder de mudar a ordem de gameplay. Assim, ligar um painel não muda o resultado do jogo.

### 3.10 Associação de entidades: identidade, posição e propriedade

Uma espada, uma fonte de áudio ou um efeito podem existir sozinhos e depois acompanhar outra entidade. São três contratos distintos:

| Relação | Efeito | Regra |
|---|---|---|
| Lógica | Referência como `owner`, alvo ou item equipado | Não altera transformação nem implica destruição conjunta |
| Espacial | Filho acompanha Transform do pai ou socket | Um pai espacial, sem ciclos, ordem pai → filho |
| Ciclo de vida | Define destino quando o pai é removido | Política explícita `detach` ou `destroy`; padrão `detach` |

API candidata: `ent.attach(child, parent, socket, opts?)` e `ent.detach(child)`. `opts` declara alinhamento e política de remoção quando diferirem dos padrões. Equipar/desequipar é operação de um kit que combina essas primitivas e regras de inventário; não exige `player:equip()` no núcleo. O vínculo espacial inicial suporta posição/offset 2D, espelhamento e sockets definidos em metadados; não cria um sistema de ossos obrigatório.

- Attach valida IDs, cena, Transform e socket. Um socket inexistente gera erro; não reposiciona silenciosamente em `(0,0)`. Ciclos são rejeitados antes de alterar a relação.
- O filho mantém identidade, controlador e configuração. Seguir a posição do Player não altera volume ou comportamento de um Speaker, nem faz a espada herdar a classe do Player.
- Attach/detach explícitos preservam posição mundial por padrão; alinhar ao socket exige opção explícita. Remover o pai com `detach` preserva a última transformação mundial válida; `destroy` agenda a remoção do filho na mesma barreira. Relações espaciais não sobrevivem à saída da cena no MVP.
- Snapshots serializam relações e políticas; `state --full` e o inspetor mostram pai/socket e referências lógicas. Testes cobrem independência, associação, separação, remoção do pai e rejeição de ciclos.

---

## 4. Agent-friendly (o centro da engine)

### 4.1 O loop do agente
```
escrever -> cycle (check + run + verificar, 1 chamada) -> why / observe se falhar -> corrigir
```
Cada etapa precisa ser rápida, barata em tokens e conclusiva.

### 4.2 Observabilidade: o agente "vê" o jogo
| Comando | Função |
|---|---|
| `engine screenshot` | PNG do frame atual |
| `engine filmstrip --frames 0,10,20,30` | **Uma imagem com N frames lado a lado** (movimento e juice em 1 imagem) |
| `engine ascii` | Frame em texto, barato em tokens |
| `engine state` | Cena e entidades nomeadas/tagueadas; resumo de componentes, controlador, relações e valores; detalhes sob demanda |
| `engine diff a.png b.png` | O que mudou, com região destacada |
| `engine log` | Eventos (colisão, spawn, morte, som) em TOON tabular; gravados em disco como JSONL |
| `engine record` | GIF/MP4 curto para o humano |

### 4.3 Verificação: provar que funciona
- **Determinismo total:** timestep fixo + RNG semeado + inputs gravados = mesma saída sempre.
- `engine replay inputs.toon --seed 42`
- `engine test --expect "player.hp==3 @frame 120"`
- Golden screenshots (hash de framebuffer) para regressão visual.
- `engine playtest`: bot aleatório por N frames, detecta crash, softlock, NaN, vazamento de entidades.
- Testes do jogo escritos em Luau, rodados em headless.
- Contratos do mundo cobrem IDs obsoletos, troca de cena, remoção durante update, dependências de componentes, entrega ordenada, limpeza de ouvintes e attach/detach. Snapshots incluem cena, componentes, estado dos controladores, relações, filas e RNG; salvar posição sozinho não basta.

### 4.4 Feedback rápido
- `engine check`: typecheck Luau + validação de assets, cenas, componentes, controladores, relações e dependências de módulos. Meta de resposta em ms, medida no projeto de referência.
- Hot reload **preservando estado compatível** (o agente edita e vê o efeito sem reiniciar). Alteração de schema pede migração declarada ou reinício explícito, com diagnóstico; nunca promete preservar dados que não sabe converter.
- `engine eval "<luau>"` em jogo rodando: inspecionar e mutar estado ao vivo.
- Headless acelerado: simular 10k+ frames/s.

### 4.5 Erros que ensinam
Todo erro é TOON com causa, local e correção:
```
error:
  code: E012
  file: player.luau
  line: 14
  msg: "gfx.rect expects 5 args, got 4"
  fix: "gfx.rect(x,y,w,h,color)"
  docs: engine docs gfx.rect
```
Vários erros (`engine check`) viram uma tabela, uma linha por erro:
```
errors[2]{code,file,line,msg,fix}:
  E012,player.luau,14,"gfx.rect expects 5 args, got 4","gfx.rect(x,y,w,h,color)"
  E031,enemy.luau,7,"unknown function gfx.box","gfx.rect(x,y,w,h,color)"
```
Regras: mensagem curta, sempre com `fix` quando possível, código estável, sem stack trace gigante por padrão.

### 4.6 Descoberta sem encher o contexto (problema principal)
O agente não deve ler docs inteiras. Camadas:

| Nível | Conteúdo | Custo |
|---|---|---|
| L0 | `llms.txt`: API inteira, 1 linha por função | ≤3k tokens |
| L1 | `engine docs <módulo>`: detalhes sob demanda | ~300-800 tokens |
| L2 | `engine examples <tema>`: código executável | ~200-500 tokens |
| L3 | `engine api` (TOON) + `.d.luau` (tipos para autocomplete) | por consulta |

Mais:
- `engine new` gera `AGENTS.md` do projeto com <1k tokens: convenções, comandos, loop recomendado.
- `engine search "<intenção>"`: "como faço screen shake?" retorna a função certa.
- Saída truncada por padrão, `--full` quando necessário.
- Nomes óbvios e consistentes (`gfx.rect`, `gfx.spr`, `input.down`, `snd.play`). Nada de sinônimos.

### 4.7 Compatibilidade com o que a IA já sabe
- Estrutura `load()` / `update(dt)` / `draw()` (LÖVE).
- Funções curtas e familiares (PICO-8: `spr`, `rectfill`, `btn`).
- Convenções de Lua padrão (tabelas, closures, `:` para métodos).
- **Estabilidade da API:** versionar e nunca quebrar sem camada de compatibilidade. Cada breaking change invalida o que a IA aprendeu.

### 4.8 Arte: o humano cria, o agente usa
**Regra:** o agente não gera arte final. Pixel art é do usuário. O agente programa com o que existe e pede o que falta.

- **Contrato por nome:** o código usa `gfx.spr("player_idle")`. O nome é a interface; o conteúdo muda sem tocar no código.
- **Placeholders automáticos:** nome inexistente vira caixa colorida com rótulo. O jogo roda e é testável desde o minuto 1.
- **`engine assets`:** lista nomes, tamanhos, frames, tags, pivôs, hitboxes. O agente "conhece" a arte sem ver pixels.
- **`engine art todo`:** o agente registra o que falta (nome, tamanho, frames, descrição) em `ART_TODO.md`. O usuário desenha no editor; ao existir o asset com aquele nome, o placeholder some.
- **Metadados são texto** (`.toml` ao lado do `.png`): animações, durações, pivôs, hitboxes. O agente ajusta timing e hitbox, nunca os pixels.
- O agente avalia o resultado via `screenshot`/`filmstrip` (ex.: sprite 16x16 com hitbox 8x8).
- Sprites ASCII só para placeholders e testes, não são o fluxo de arte.

### 4.9 Segurança e robustez
- Sandbox Luau: sem `io`, `os`, rede.
- Orçamento de tempo por frame (script lento não trava o loop).
- Projeto isolado na própria pasta.
- Erros de script são contidos na VM: diagnóstico + frame anterior mantido, simulação pausada no tick que falhou. Após corrigir, retomar de snapshot/tick confirmado ou reiniciar a cena; hot reload não autoriza continuar um tick parcialmente executado. Limites da VM não prometem recuperar falhas fatais de código nativo.

### 4.10 MCP: poucas tools, orientadas a workflow
O MCP é um wrapper fino do mesmo core do CLI, mas suas tools são **workflows**, não um comando por tool.

**Tools (≤8):** `cycle`, `observe` (parâmetro `view`: screenshot | filmstrip | state | ascii), `why`, `eval`, `docs`, `snapshot`, `art_todo`.

- **Workflow, não endpoint:** `observe` com `view` substitui 4 tools. Menos schema, menos escolha errada.
- **Descoberta progressiva:** descrição de 1 linha (<100 chars); detalhe via tool `docs`. Tools de plugins só aparecem se o plugin está ativo, agrupadas por categoria.
- **Code mode:** `eval` e `cycle` executam script/filtro no servidor e só o resumo volta ao contexto. Logs e estado grandes nunca passam inteiros.
- **Annotations:** `observe`, `why`, `docs` com `readOnlyHint` (aprovação automática possível); ações destrutivas (`snapshot restore`) marcadas.
- **Resources e prompts:** `llms.txt`, `AGENTS.md`, `api` e plugins ativos como *resources*; receitas (`new-game`, `fix-failing-test`, `add-juice`) como *prompts*. Não gastam slot de tool.
- **Orçamento:** toda tool aceita `budget` (tokens). O servidor trunca e devolve `truncated: true` + cursor.
- **Imagens baratas:** framebuffer low-res custa pouco em visão. Um filmstrip de 4 frames 320×180 fica na ordem de ~300 tokens (estimativa). Sempre um mosaico, nunca N imagens.
- **Sessão persistente:** o servidor mantém o jogo rodando com hot reload; sem reboot por chamada.
- **Instalação:** `engine mcp install <claude-code|cursor|codex|...>` configura o MCP e gera skill/`AGENTS.md`/rules do agente escolhido.
- Respostas em TOON (4.12). Plugins podem registrar tools.

### 4.11 Métricas de sucesso (como saber se é referência)
- Tokens até o primeiro jogo jogável com agente zero-shot.
- Número de iterações (check -> run -> fix) até corrigir um bug.
- % de tarefas concluídas sem o agente ler docs L1+.
- Tempo de um ciclo completo (editar -> ver resultado).
- Benchmark público: "faça um jogo X" rodado em vários modelos.
- Tokens por saída do CLI (TOON vs JSON), medidos nas saídas reais (`state`, `check`, `log`).
- Chamadas (idas e voltas) por tarefa e tokens de contexto por sessão.
- Orçamentos de saída (4.14) verificados no CI por `xtask token-budget`.

### 4.12 Formato de saída: TOON em vez de JSON
O agente **lê** a saída da engine, então o formato é custo direto de contexto. TOON (Token-Oriented Object Notation) declara as chaves uma vez e lista os itens em linhas, o que reduz tokens em dados uniformes (arrays de objetos: entidades, eventos, erros, assets, o caso dominante aqui). Spec v3.0, com crate Rust oficial (`toon-format`, baseado em serde).

JSON:
```json
{"frame":120,"entities":[{"id":1,"tag":"player","x":48,"y":120,"hp":3},{"id":2,"tag":"enemy","x":200,"y":120,"hp":1},{"id":3,"tag":"enemy","x":240,"y":100,"hp":2}]}
```
TOON (`engine state`):
```
frame: 120
entities[3]{id,tag,x,y,hp}:
  1,player,48,120,3
  2,enemy,200,120,1
  3,enemy,240,100,2
```
O ganho cresce com o número de linhas (as chaves não se repetem).

**Regras:**
- Toda saída estruturada do CLI e do MCP é TOON por padrão. `--format json` (e `text`) continuam existindo para CI e ferramentas externas.
- Implementado como plugin `Formatter` (`fmt-toon`, `fmt-json`): o core não sabe de formato (SOLID).
- **TOON é para o agente ler.** Para o agente **escrever** dados, ele usa Luau, flags e TOML, formatos que os modelos dominam melhor.
- Divisão de formatos: TOON = saídas e dados tabulares grandes (replays, logs lidos); TOML = o que humanos/agentes editam (config, manifests, VFX, metadados); PNG = pixels.
- Replays são `inputs.toon`, tabela `inputs[N]{frame,key,action}:`. Gravação em JSONL durante a sessão, convertida ao fechar (TOON declara o tamanho `[N]`, ruim para append).
- Logs append-only em disco ficam em JSONL; `engine log` lê e exibe em TOON.
- Sempre usar o encoder, nunca montar TOON à mão (quoting e delimitadores: vírgula, tab ou `|`).
- O L0 (`llms.txt`) e o `AGENTS.md` incluem um primer de 3 linhas sobre TOON.
- Dados não uniformes ou muito aninhados podem sair maiores em TOON: medir (métrica 4.11) e, se necessário, achatar o formato da saída.

### 4.13 Menos idas e voltas: pipeline e diagnóstico

**`engine cycle [--frames N] [--expect "..."]`**: check → run headless → asserts → resumo, em 1 chamada. Silêncio no sucesso:
```
cycle: ok
```
Na falha, só o que importa:
```
cycle: fail
failed[1]{test,frame,expected,actual}:
  enemy_hp,90,1,2
shot: out/cycle.png
```

**`engine why <pergunta>`**: diagnóstico de domínio, causas ranqueadas com `fix`.
- `why invisible player`: fora da câmera? sprite faltando (placeholder)? camada atrás do tilemap? alpha 0? entidade morta?
- `why not-moving enemy_2`: sem input/IA? colisão bloqueando? estado errado?
- `why no-sound hit`: som não registrado? volume 0? canal cheio?

**Erros com "did you mean":** `entity 'enemy1' não existe; existentes: enemy_1, enemy_2`. Vale para entidades, assets, funções e eventos.

**Snapshots e rewind:** `engine snapshot save|restore <nome>`, `engine rewind <frames>` (usa determinismo + log de inputs). Commit git automático por ciclo bem-sucedido, em branch `engine/auto` (opt-out). O agente experimenta sem medo de quebrar.

**Invariantes declarativas** (`invariants.toml`), checadas a cada frame em `run`/`playtest`:
```toml
[[invariant]]
expr = "player.hp >= 0"
[[invariant]]
expr = "ent.count('bullet') < 200"
```

**Lints de erros comuns de agente** no `check`: alocação no `update`, `dt` nulo, NaN, tabela criada por frame no `draw`, global acidental. Cada lint traz `fix`.

**Telemetria local opt-in** (`engine telemetry on|off|export`): registra em arquivo local os erros mais frequentes e ciclos por tarefa (TOON). Nada é enviado automaticamente; o export é manual. Serve para ajustar API e mensagens com dados reais.

### 4.14 Economia de tokens e de contexto

**Saída:**
- Silêncio no sucesso (`ok`), detalhe só na falha.
- Delta: `state --since last`, `log --since last` devolvem só o que mudou.
- Truncamento com contagem: `+47 itens (--full ou --page 2)`.
- Orçamento: `--budget N` em todo comando (padrão 500 tokens); trunca e resume.
- Saídas **determinísticas e estáveis**: sem timestamp, ID aleatório ou ordem instável. O prefixo do `llms.txt` é imutável. Isso permite cache de prompt no lado do agente.
- Formato TOON (4.12).

**Contexto:**
- `engine map`: mapa do projeto, 1 linha por arquivo, no lugar de ler os arquivos:
```
files[3]{path,lines,exports}:
  main.luau,48,"load update draw"
  entities/player.luau,92,"Player.new Player.update"
  systems/combat.luau,140,"Combat.hit"
```
- `engine status`: 1 linha. `ciclo 14 | check ok | 2/5 testes | tarefa: inimigos atiram`.
- `engine resume`: resumo da sessão gerado do log de eventos (tarefa atual, últimos ciclos, falhas abertas, arquivos alterados). Reorienta o agente após compactação de contexto ou numa sessão nova.
- Arquivos ≤300 linhas e módulos independentes: o agente lê só o necessário e subagentes trabalham em paralelo sem conflito.

**Orçamentos-alvo (verificados no CI):**

| Saída | Alvo |
|---|---|
| `llms.txt` (L0) | ≤3k tokens |
| `cycle` ok / fail | ≤10 / ≤200 |
| Erro individual | ≤150 |
| `state` (padrão) | ≤500 |
| `map` / `resume` | ≤300 / ≤400 |
| `filmstrip` (4 frames) | ≈300 (imagem) |

### 4.15 Facilitar a codificação do agente
- **Kits de gênero (plugins Luau):** platformer, top-down, shmup, puzzle. `engine new --template platformer`. Vêm com componentes prontos (controller, câmera, inventário, diálogo, spawner). O agente compõe e configura em vez de escrever do zero.
- **Declarativo antes de imperativo:** cenas/presets em TOML, componentes como dados, controladores Luau pequenos e simulação repetitiva em systems Rust. Exemplo: o kit instancia `npc` com Sprite `enemy_walk`, vida 2 e controlador `patrol`; Player e NPC reutilizam o mesmo movimento. Ver contrato de cena em 3.7.
- **Composição verificável:** `check` explica componente ou módulo faltante; `state` mostra o que está ativo; `why` identifica controlador ausente, destino removido, socket inválido e mensagem sem receptor. Kits incluem cenários de teste, não apenas um conjunto de presets.
- **Convenção sobre configuração:** pastas convencionais (`entities/`, `controllers/`, `systems/`, `scenes/`, `assets/`, `plugins/`), criadas só quando usadas. `entities/` guarda presets TOML; `controllers/` e `systems/` de projeto guardam Luau. Um único jeito de fazer cada coisa, sem sinônimos.
- **`engine scaffold system|entity|controller|scene <nome>`:** gera Luau para comportamento e TOML para dados, com schema/tipos, TODOs e registro no projeto.
- **Tipos e LSP prontos:** `.d.luau` e config do `luau-lsp` gerados no `engine new`; modo strict no `check`. Erros chegam antes de rodar.
- **Exemplos testados no CI:** todo exemplo de `engine examples` roda com `--expect`. A doc nunca desatualiza.
- **Busca por intenção:** `engine search "screen shake"` retorna a função e um exemplo.

---

## 5. Referência rápida de comandos (CLI)

```
engine new <nome> [--template platformer|topdown|shmup]   # projeto + AGENTS.md + tipos/LSP
engine check                   # typecheck + lints + validação, ms
engine cycle [--frames N] [--expect "..."]   # check + run + verificar, 1 chamada
engine why <pergunta>          # ex.: why invisible player
engine run [--headless] [--frames N] [--seed S]
engine dev                     # viewer com hot reload e overlay
engine screenshot [--frame N]
engine filmstrip --frames 0,10,20
engine ascii [--frame N]
engine state [--filter tag:enemy]
engine eval "<luau>"
engine replay <inputs.toon>
engine test [--expect "..."]
engine playtest [--frames N]
engine diff <a.png> <b.png>
engine docs <tópico> | examples <tema> | search "<intenção>"
engine api
engine live [--theme <t>] [--calm]
engine plugin add|rm|list|info <nome>
engine snapshot save|restore <nome> | rewind <frames>
engine map | status | resume
engine scaffold system|entity|controller|scene <nome>
engine mcp [install <agente>]
engine telemetry on|off|export
engine edit sprite|vfx <nome>
engine build [--target linux|windows]   # bytecode Luau + .pak
engine profile [--frames N]    # ms por sistema, FFI, alocações
engine bench [--frames N]      # headless determinístico

# flags globais: --format toon|json|text, --budget N, --since last, --full
```

---

## 6. Editores embutidos (arte e VFX)

Motivo: não trocar de programa. Mesmo binário (`engine edit`), mesmo projeto, **hot reload no jogo rodando** enquanto o usuário pinta.

### 6.1 Editor de pixel art (Aseprite enxuto)
- Ferramentas: lápis, borracha, balde, linha, retângulo, elipse, seleção, mover, conta-gotas, simetria
- Camadas, frames, onion skin, timeline com tags de animação
- Paleta do projeto (a mesma do jogo), ramps, troca de paleta
- Slices: pivô, hitbox, 9-slice
- Modo tileset/tilemap com autotile básico
- Preview em escala real e dentro do jogo
- Atalhos no estilo Aseprite, desfazer ilimitado, zoom e grid
- **Formato:** `.png` (pixels) + `.toml` (metadados). Diff legível no git.
- **Fora do escopo v1:** pincéis complexos, filtros, importar PSD.

### 6.2 Editor de VFX
- Emissores de partículas: vida, velocidade, gravidade, forma, rotação, burst/contínuo
- Curvas ao longo da vida (tamanho, alpha, velocidade) e gradiente de cor da paleta
- Efeitos de tela: shake, flash, hitstop, slow-mo, com curvas
- Editor de curvas de tween/easing
- Preview ao vivo em loop, com disparo manual
- **Formato:** `.toml`. No código: `juice.fx("explosion", x, y)`.
- VFX é **parâmetro, não pixel**: o agente cria variações e ajusta números em texto; o humano refina visualmente. Edição compartilhada.

### 6.3 Editor de cenas e inspetor (depois do jogo de referência)
O fluxo humano complementar é: abrir uma cena → inserir um preset → configurar componentes → associar assets/controlador → conectar relações e mensagens → executar e inspecionar. O agente faz as mesmas operações nos arquivos e pelo CLI.

- Árvore/lista de entidades com busca por nome/tag, viewport e inspetor de componentes, controlador e relações. Entidades sem sprite continuam selecionáveis como marcadores.
- Presets dos kits reduzem montagem repetitiva. O inspetor só oferece capacidades disponíveis e mostra dependências antes de adicionar um componente.
- O schema que gera tipos e validação também descreve os campos editáveis; evitar um segundo cadastro manual de propriedades.
- Salvar produz o mesmo TOML, com ordenação estável, e passa pelo mesmo `check`. Alterações ao vivo respeitam as barreiras estruturais; persistir uma mudança do runtime para a cena é uma ação explícita.
- Editor não é necessário para criar, rodar, testar ou construir o jogo. Tilemaps, state machines e diálogos vêm depois, conforme uso real. Lógica continua em Luau.

### 6.4 Implementação
- Crates `editor-*` atrás da feature `editor`. O runtime do jogo continua <5MB; o editor é binário/feature separada.
- Editor, CLI e jogo compartilham o crate `assets` (um parser, uma fonte da verdade).
- `engine edit sprite player` abre direto no asset.
- **UI, decisão em aberto:** (a) `egui`: rápido de construir, custo só no editor; (b) dogfooding: editor escrito na própria engine em Luau, com resolução de UI maior (prova a API, mais lento de fazer). Recomendo (a) no MVP.

---

## 7. Juiciness

Dois níveis: o juice **do jogo** (7.1) e o juice **do processo**, ou seja, ver o agente trabalhando em tempo real de forma clara e satisfatória (7.2). O segundo é o diferencial da engine.

### 7.1 Juice do jogo
Primitivos de 1 linha, no módulo opcional `juice`:
```lua
juice.shake(4, 0.2)
juice.hitstop(0.08)
juice.tween(e, "x", 100, 0.3, "outback")
juice.squash(e, 1.3, 0.7)
juice.burst(x, y, "spark", 12)
juice.fx("explosion", x, y)      -- efeito do editor de VFX
```
Easings prontos, partículas em Rust (batch), flash, trails, números flutuantes. Efeitos são dados `.toml` criados no editor de VFX (6.2). O agente verifica o juice via `filmstrip` e `log`.

### 7.2 Agent Live View
**Objetivo:** cada ação do agente vira feedback visual e sonoro imediato, **legível de relance** e prazeroso de acompanhar.

**Fonte única:** o barramento `events` de observação (3.9). Ações registradas pela engine emitem um evento tipado (`file_changed`, `reload`, `check_ok`, `check_fail`, `entity_spawned`, `state_changed`, `test_pass`, `test_fail`, `task_done`...). O Live View só assina eventos (é plugin de `Panel` + `Fx`); não conhece o agente nem o jogo.

| Evento | Feedback |
|---|---|
| `file_changed` | Arquivo entra no feed lateral com ícone e mini-diff; moldura do jogo pisca amarelo |
| `reload` | Varredura de luz na moldura, "tick" sonoro; o estado do jogo é preservado |
| `check_ok` / `check_fail` | ✓ verde com pop / ✗ vermelho com shake curto e a linha do erro no feed |
| `entity_spawned` | Pop-in (escala + flash + partículas) e rótulo com o nome por 1s |
| `state_changed` | Valor pulsa no HUD, número sobe/desce com cor |
| `sprite_missing` | Placeholder pisca com o nome pedido e entra em `art todo` |
| `test_pass` / `test_fail` | Barra verde varre e o combo sobe / vermelho com expected vs actual |
| `task_done` | Celebração (partículas, jingle) e a barra de progresso do backlog avança |
| `playtest` | Bot visível, trilha de posições, heatmap de onde morreu |

**Layout:**
- **Jogo no centro**, nunca coberto.
- **Painel lateral:** feed de ações (últimas N, ícones, cores semânticas), tarefa atual, progresso do backlog.
- **Timeline inferior:** eventos por frame; clicar = scrub via replay determinístico.
- **Agent marker:** colchetes + rótulo na entidade/região que o agente está mexendo agora.
- **Contadores:** sequência de checks verdes, testes passando, ciclos por minuto.
- **Time-lapse:** reconstruir a sessão inteira a partir do log de eventos (possível pelo determinismo).

**Regras de design (clareza antes de dopamina):**
1. Todo efeito carrega informação. Cores semânticas fixas: verde = ok, vermelho = erro, amarelo = mudança, azul = novo.
2. Curto: ≤400ms por efeito, empilhamento limitado.
3. Nunca cobre a área do jogo; efeitos globais só na moldura.
4. Som opcional, chiptune sintetizado, um timbre por categoria de evento.
5. `--calm` reduz movimento; temas acessíveis (contraste).
6. Mesma linguagem visual para todos os eventos: aprender uma vez.
7. Custo ~0 quando desligado: é plugin.

**Tudo plugável:** painéis, temas (`.toml`), efeitos e sons por evento. `engine live --theme neon`.

---

## 8. Esboço do `llms.txt` (L0, ~3k tokens no final)

```
# ENGINE v0 — 320x180, paleta 32 cores, Luau
# Callbacks: load() update(dt) draw()
# Loop: engine cycle (1 chamada) | dúvida: engine why <x> | contexto: engine map, resume
# Saída do CLI é TOON: `chave: valor`; listas = `nome[N]{campos}:` + 1 linha por item

gfx.clear(c)                     limpa tela
gfx.rect(x,y,w,h,c)              retângulo preenchido
gfx.rectb(x,y,w,h,c)             contorno
gfx.spr(id,x,y,[flipx],[flipy])  desenha sprite
gfx.text(s,x,y,c)                texto
gfx.cam(x,y)                     câmera

input.down(k)  input.pressed(k)  input.axis(neg,pos)

snd.play(name,[vol])

ent.new(tag,[x],[y]) -> e        cria entidade; coordenadas adicionam Transform
ent.each(tag) -> iter            itera por tag
ent.kill(e)
# Propostas adicionais; assinaturas finais após validar no jogo de referência:
scene.load(name)                agenda transição validada
ent.attach(child,parent,socket,[opts]) associa posição; identidade preservada
ent.detach(child)               separa preservando posição mundial
msg.send(target,name,payload)   mensagem para o próximo tick
evt.emit(name,payload)          evento para o próximo tick

phys.overlap(a,b) -> bool
phys.sweep(e,dx,dy)              move com colisão

juice.shake(amp,t)  juice.hitstop(t)  juice.tween(o,k,to,t,ease)
juice.burst(x,y,kind,n)

rnd(n) / rnd_seed(s)             RNG determinístico
log(tag,data)                    evento p/ engine log
```
Mais: `engine docs <função>` para detalhes.

---

## 9. Roadmap

| Fase | Entrega e critério de saída |
|---|---|
| 1. Núcleo executável | Contratos mínimos + composition root, ECS/SoA com IDs geracionais e barreiras de ciclo de vida, cena implícita/uma cena TOML, loop fixo, Luau, renderer software, PNG + placeholders, `run --headless`, `screenshot`, `check` básico e erros estruturados. Aceite: entidade criada, atualizada e removida sem handle obsoleto aceito; frame reproduzível com seed fixa |
| 2. Observar e reproduzir | Input/janela, `events`, TOON, `state`, `log`, `filmstrip`, `ascii`, replay e baseline de `profile`/`bench`. Aceite: mesmas entradas produzem os mesmos hashes/estado; observação ligada/desligada não altera o resultado |
| 3. Jogo de referência por agente | `cycle`, `test --expect`, `llms.txt`/`AGENTS.md` mínimos, controlador único, mensagens/eventos ordenados, relações attach/detach, presets mínimos e troca de cena. Um jogo curto: Player e NPC compartilham movimento, espada independente pode ser equipada/solta, coleta abre porta por relay, mudança de cena limpa recursos; Speaker opcional acompanha o Player. Aceite: agente cria e corrige pelo CLI com placeholders; testes verificam comportamento e ciclo de vida |
| 4. Contratos, diagnóstico e módulos | `check` completo + lints, `why`, schemas de componentes/mensagens, snapshots/rewind completos, invariantes, golden hashes Linux/Windows, registry Rust e host Luau com dependências/versionamento. Aceite: dependência ausente/cíclica e destino removido geram diagnóstico; restaurar snapshot reproduz a continuação |
| 5. Contexto e Live View MVP | Docs/exemplos gerados, `--budget`/delta, `map`/`status`/`resume`, `xtask token-budget`, MCP MVP, feed e feedback visual de reload/check/test, telemetria local opt-in. Aceite: repetir as tarefas do jogo de referência com orçamento de contexto medido |
| 6. Juice, kits e performance medida | `juice` e áudio opcionais, sidecars TOML, `assets`/`art todo`, kits de gênero com controladores/presets, `scaffold`, `new --template`, ponte por buffers, GC com orçamento e `xtask perf-budget`. Código nativo/SIMD só onde os perfis justificarem. Aceite: kit melhora o jogo de referência sem obrigar outros projetos a carregar suas capacidades |
| 7. Editor de pixel art | MVP da seção 6.1 com hot reload. Aceite: trocar placeholder por arte humana sem mudar gameplay |
| 8. Editor de VFX + Live View completo | Seção 6.2, temas, timeline, time-lapse e sons. Aceite: efeitos configurados em TOML reproduzíveis por seed/frame |
| 9. Editor de cenas/inspetor + MCP completo | Seção 6.3, resources/prompts/annotations/code mode; demais editores só conforme demanda. Aceite: cena editada visualmente continua legível, validável e executável só pelo CLI |
| 10. Distribuição | Builds Linux/Windows, bytecode e `.pak`, pacote de projeto e site. PGO/variantes SIMD conforme medição. Aceite: projeto empacotado mantém testes/replay e não carrega módulos desativados |

**Regra:** cada fase termina com um fluxo executável e verificável. O jogo de referência chega na fase 3, antes de um framework amplo de plugins ou editores. Testes de contrato e disciplina arquitetural acompanham cada entrega; metas de tamanho, memória e velocidade são medidas por perfil de build e hardware declarado.

---

## 10. Backlog e organização (gratuito, minimalista)

- **Backlog.md**: tasks em markdown dentro do repo, CLI + kanban no terminal, agent-friendly. Verifique se o MCP atual atende.
- **GitHub Issues + Projects**: grátis, zero setup, bom para contribuidores externos.
- Convenção: 1 task = 1 arquivo, critério de aceite verificável por comando (`engine test ...`).
- Recomendo começar com Backlog.md (a própria engine é desenvolvida com agentes) e abrir Issues quando houver comunidade.

---

## 11. Riscos

| Risco | Mitigação |
|---|---|
| "Linux das engines" é meta de longo prazo | Vertical slice primeiro, escopo mínimo |
| Agente não conhece a API | Imitar LÖVE/PICO-8, docs em camadas, `search` |
| API cresce e vira inchada | Orçamento de ~60 funções, remover antes de adicionar |
| Software renderer lento em escalas maiores | Resolução baixa por design; backend GPU como feature futura |
| TOON é menos familiar aos modelos que JSON | Primer de 3 linhas no L0, agente só lê TOON (escreve Luau/TOML), `--format json` como fallback, medir tokens e acertos |
| Features agent-friendly incham o CLI (contradiz o minimalismo) | Tudo que dá para ser plugin é plugin; comando só entra com métrica de ganho (menos chamadas ou tokens); `token-budget` no CI |
| Snapshots e commit automático geram ruído ou perda | Branch `engine/auto`, opt-out, snapshots limitados em pasta ignorada |
| Kits de gênero viram uma engine dentro da engine | Presets/controladores opcionais, pequenos e testados; inventário/equipamento fora do core |
| Modelo de atores duplica ECS ou cresce por herança | Uma identidade `ent`, componentes SoA e presets; Player/NPC compartilham dados e trocam controlador |
| Mensagens viram ciclos ou custo imprevisível | Entrega por tick, ordem estável, schemas e limites; origem/cadeia causal nos diagnósticos |
| Relações deixam filhos, ouvintes ou handles órfãos | Política detach/destroy explícita, limpeza por escopo e testes de ciclo de vida |
| Múltiplos controladores disputam o mesmo estado | Um controlador por entidade no MVP; composição interna explícita |
| Cena/editor cria outra fonte da verdade | Mesmo TOML, schema, carregador e validador no CLI e no editor |
| Hot reload perde estado ou preserva dados incompatíveis | Estado serializável separado do código; migração declarada ou reinício explícito |
| Código nativo do Luau (`luau-jit`) é experimental | Opcional, por função, medido; interpretador como fallback |
| Ponte por buffers complica a API para o agente | Escondida atrás de funções simples (`ent.pos`, `ent.set_vel`); o agente nunca vê buffers |
| Otimização prematura contra o minimalismo | Só otimiza o que o `profile` apontar; `perf-budget` como guarda |
| Float diverge entre Linux e Windows (replay quebra) | `libm`/ponto fixo, sem fast-math, golden hashes nos dois SOs no CI |
| Raster multithread quebra determinismo | Tiles disjuntos, desligado por padrão |
| Divergência Luau/Rust (bindings) | Gerar `.d.luau` e docs a partir do código Rust (fonte única) |
| Breaking changes quebram o que a IA sabe | Versionamento estrito, camada de compatibilidade |
| Abstração excessiva (SOLID vira burocracia) | Trait só em fronteira real; módulos internos antes de novos crates; validar arquitetura no jogo de referência |
| Plugins quebram a API ou incham o contexto do agente | Faixa compatível e revisão fixada, dependências validadas, L0 só com ativos; falha obrigatória pausa com diagnóstico |
| Live View vira distração ou gasta CPU | Regras de design 7.2, é plugin desligável, `--calm` |
| Editor de pixel art vira projeto maior que a engine | Lista fechada de ferramentas (6.1); implementação depois do jogo de referência e das ferramentas básicas |
| Arte atrasa e o jogo fica feio de placeholder | Placeholders legíveis, `art todo` priorizado, editor com hot reload para iterar rápido |

---

## 12. Princípios finais (para pendurar na parede)

1. **Texto é a interface.** Tudo é legível e editável por agente e humano.
2. **Observar, verificar, corrigir.** Nunca pedir ao agente que adivinhe.
3. **Menos é mais.** API pequena, restrições fixas, contexto curto.
4. **Determinismo é feature.** Sem ele não há teste, replay nem confiança.
5. **Rust executa, Luau decide.**
6. **CLI é a verdade.** GUI e MCP são conveniências.
7. **Nunca quebre o que o agente já aprendeu.**
8. **Arte é do humano.** O agente programa e pede; o humano desenha, no mesmo programa.
9. **Núcleo pequeno, capacidades opcionais.** Contratos SOLID, composição e dependências explícitas; abstrações só onde ajudam.
10. **O processo também é produto.** Ver o agente trabalhar deve ser claro e satisfatório.
11. **Cada token de contexto é do agente.** Saída em TOON, docs em camadas, nada de ruído.
12. **Menos idas e voltas.** Uma chamada, um resumo. Silêncio no sucesso, detalhe só na falha.
13. **Rápido por design, provado por medição.** Luau declara, Rust simula; orçamento de frame, zero alocação no hot path, `perf-budget` no CI.
14. **Entidades são composição, não subclasses.** Identidade única, componentes opcionais, controladores reutilizáveis.
15. **Relação não é propriedade implícita.** Seguir posição, possuir logicamente e destruir junto são decisões separadas.
16. **Um jogo completo valida a arquitetura.** Editor, plugins e otimizações crescem a partir de necessidades observadas.

---
