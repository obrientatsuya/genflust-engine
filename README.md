# genflust-engine
    GENFLUST - ENGINE 2D AGENT-FIRST 

    visao geral

    rust core + luau scripting. pixelart/2d, open source
    full modular, CLI first

    objetivo -> engine extremamente veloz e eficiente desenhada para ser a melhor operada por agentes. (problema existente, porém soluções integram e não são nativas.)

0. TL;DR


pilar / decisao

EFICIENCIA / renderer por software em framebuffer low-res, command buffer, efi em batch, binario extremamente leve.

MINIMALISMO / (NAO PARECER UMA CABINE DE AVIAO aka UNITY) api de ~60 funções, resolução e paleta (cores)fixas, projeto = pasta pequena

CLAREZA VISUAL / pixel-perfect, debug ovelay, filmstrip, magicas do UI / UX design rsrsr

CLI / toda funcionalidade é um comando, GUI e MCP sao wrappers.

LUAU + RUST / rust executam, luau decide. luau pelo typecheck estatico e sandbox nativo (melhor pra agents e eRROS)

art / humano desenha, editor de pixelart e vfx incluso, agente usa por nome e placeholders.

MODULARIDADE / solid estrito, tudo é plugin, rust estatico + luau runtime, atrás de traits pequenas

JUICINESS / live view, ver oq o agente ta fazendo exatamente em tempo real claro e dopaminergico. (trabalhe imediatamente iA maldita)

SAÍDA / TOON PADRÃO POR ECONOMIA DE TOKEN, USAR JSON EM DADOS UNIFORMES (O MENOS POSSIVEL)

PERFORMANCE / LUAU DECLARA E RUST SIMULA, ECS/SoA, ZERO alocação, raster indexado + SIMD; medido no CI (APROFUNDAR)

CONTEXTO / deve ser o mais economico possivel, silencio em sucesso, delta, --budget por saida, cycle, map, resume.

AGENT-FRIENDLY / observar, verificar, corrigir sem adivinhar, contexto minimo api conhecida por llms.txt




1. analise em primeira instancia

1 - agente nao conhece a engine (sem treinamento), : imitar apis existentes (nomenclaturas) like: LOVE (load,update,draw) ou PICO-8(opensource) 

2 - agent friendly x minimalismo : é contraditorio pq um pede mt ferramenta e o outro nao permite, entao uma unica superficie -> cli, poucos comandos, saida preivsivel, composaveis, mcp = wrapper fino.

3 - eficiencia x observabilidade : renderer por software deterministico barato em 320x180, SEM GPU, SCREENSHOts identicos em qualquer SO.

4 - contexto food : docs em camadas (l1,l2,l3), saidas truncadas por padrao, erros estruturados com a correção embutida.

5- live view: cada ação do agente vira feedback visual e claro em tempo real, assinando baramentos de eventos

6 - tudo é full modular, extensivel sem inchar core: solid estrito, core só trais (api), toda implementaç~~ao de feature é plugin injetavel no app.

7 - input e output do agent (economizar tokens) : ´engine cycle´ em 1 request, silencio no sucess, deltas e orçamentos de tokens por saída.

8 - velocidade extrema vs simplicidade para o agent e determinismo : 
a otimização vem do rust, a api luau continua simples (ent.pos(e), ) buffers e simd sao invisiveis, matematica deterministica (libm/pointo fixo), e golden hash entre sistemas.

principio guia - cada feature precisa responder:  agent consegue usar, verificar e consertar isso so com texto e poucos tokens?

2. bostinha aprofundada por IA dos seis pilares:

Eficiência:

Update fixo a 60Hz, render desacoplado.
Zero alocação no hot path. Command buffer pré-alocado, esvaziado 1x por frame.
Nunca 1 chamada FFI por entidade. Sempre batch.
Handles inteiros (sprite id, sound id) na fronteira Rust/Lua, nunca strings/tabelas.
Meta: binário <5MB, boot <100ms, idle <30MB RAM, headless >10k frames/s. Detalhes e orçamentos em 3.6.
Cargo: lto="fat", codegen-units=1, opt-level=3, panic="abort", strip=true.

Minimalismo:

Resolução interna fixa (padrão 320×180), escala inteira.
Paleta fixa (padrão 32 cores, trocável por arquivo de texto).
API pública pequena (~60 funções). Adicionar função exige remover ou justificar.
Projeto = pasta pequena: main.luau, assets/, engine.toml, AGENTS.md.
Editores embutidos, enxutos e opcionais (feature editor), todos falando PNG + texto simples.

Clareza visual

Pixel-perfect: sem filtro, sem subpixel, escala inteira.
Paleta limitada força legibilidade.
Debug overlay: hitboxes, IDs, grid, FPS, estado de entidades selecionadas.
Tudo que o agente "vê" tem versão para humano (screenshot, filmstrip, viewer).

CLI-first? 

Todo comando funciona sem janela.
Saída estruturada em TOON por padrão, --format json para interoperabilidade, --full para detalhes.
Exit codes significativos. Idempotente quando possível.
GUI/viewer e MCP chamam o mesmo core que o CLI.



3. Luau + Rust
Loop, janela, input, áudio /	Regras do jogo
Renderer, sprite batch, tilemap, paleta	/ Comportamento de entidades
Colisão (spatial hash), partículas, pathfinding	State machines, decisões de IA
Tween/easing core, câmera, shake /	UI, diálogos, level scripts
Assets, atlas, serialização	/ Combinação de módulos
CLI, headless, replay, hot reload, MCP / Testes do jogo

4. aaaaaaaaaaaarquiteturaaaaaaaaaaaaaaaaaaaaaaaa

4.1 renderer por software screenshots bit-identicos entre linux e ruuiindows (testes de regressão confiáveis)

janela so faz upload do framebuffer como textura (softbuffer ou miniquad)

menos código, menos bugs, binario menor.

4.2 fluxo frame = 
input -> [Luau update(dt)] -> comandos (Input)
      -> [Rust] física/colisão/partículas/tween (batch)
      -> [Luau draw()] -> command buffer
      -> [Rust] raster -> framebuffer -> janela | PNG | hash

4.3 workspace cargo, nenhum arquivo passa de 300 linhas (alvo 200)
um proposito por modulo funções pequeninas

ai slop example (verificar depoisssssssss): 

crates/
  api/          # SÓ traits e tipos: Renderer, InputSource, AudioSink,
                #   AssetLoader, Command, Panel, Fx, EventSink, Formatter. Zero implementação
  events/       # barramento de eventos tipado (base de Live View, log, replay, MCP)
  core/         # loop, tempo, estado, RNG semeado
  gfx/          # impl Renderer (software, indexado): paleta, sprites, tilemap, texto
  input/        # impl InputSource, gravação/replay
  audio/        # impl AudioSink
  ecs/          # entidades SoA, IDs com geração, buffers compartilhados com Luau
  systems/      # systems embutidos por dados: movimento, gravidade, animação, vida útil
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
  mcp/          # wrapper fino sobre o CLI
  app/          # composition root: único lugar que conhece impls concretas
kits/           # plugins Luau oficiais (platformer, topdown, shmup), fora do core
xtask/          # arch-check: linhas, grafo, contratos | token-budget: orçamentos de saída | perf-budget: fps e ms por sistema

Feature flags: window, audio, mcp, live, editor. Build mínimo = api + events + core + gfx + script + cli + app.


5. COMO APLICAR SOLID? E AGORA PORRA?


S	um modulo, ≤300 linhas por arquivo - xtask arch-check falha o CI

O	estender = adicionar plugin/impl. nunca editar o core -	todo ponto de extensão passa por um registry

L	qualquer impl de uma trait substitui outra sem o core notar (ex.: renderer software x GPU) -	suite de testes de contrato por trait, rodada em todas as implw

I	traits pequenas, uma capacidade cada, menos q5 metodos. nada de trait "PIROCUDA" - arch-check conta métodos

D	core depende only da api. impls são injetadas no app - grafo de crates: só app importa impls

BONUS::::::::::::::::::
sem globals nem singletons, dependêecias entram somente por construtor. miodulos conversam por events nao por chamadas
ciclos de dependencia proibidos, cada crate é testável isolado com fakes das traits.

as regras valem para quem contribui (humano ou agente): ficam no AGENTS.md do repo da engine e sao verificadas por xtask arch-check

6. plugins: tudo que quiser carregar
ex: backend de render, audio, input, formato de asset, formato de saída (formatter), comando de CLI, tool de MCP, ferramenta de editor, painel/tema do Live View, efeito de VFX, módulo de gameplay, validador do check, gerador de docs

# plugins/platformer/plugin.toml
[plugin]
name = "platformer"
version = "0.1.0"
api = "^0"                  # versão da API da engine
requires = ["physics"]

[permissions]
fs = "project"              # none | project

[docs]
l0 = "platformer.move(e,dx)  anda com gravidade e colisão"

plugins tambem sao agent-friendly:

conferir aislop abaixooo:? 

Trazem l0, docs/, examples/ e .d.luau. Entram nas camadas L1/L2/L3
automaticamente.
O L1 (llms.txt) só lista plugins ativos: o contexto cresce apenas com o que se usa.
engine plugin add|rm|list|info. Instalação por pasta ou URL git.
Erro de plugin sempre diz o nome do plugin. Plugin com defeito é desativado + evento; o core segue rodando.
API versionada: plugin declara api = "^0"; breaking change só em major.

7. performance validada com benchmark (CUSPIDO POR IA SSSSSSSSSSSSSSSSSSSSSSSSEXO0)

7.1 Metas a validar: 

Métrica	/ Meta
Boot / <100ms
Frame de jogo típico (~1000 entidades) /	<2ms de CPU (orçamento total: 16,6ms a 60Hz)
10k entidades em systems Rust	/ <2ms
Chamadas FFI Rust↔Luau por frame /	<200
Alocações no hot path	/ 0
Headless /	>10k fps (jogo simples)
Input → tela /	≤1 frame
Pausa de GC do Luau /	<0,5ms por frame
Memória idle / binário /	<30MB / <5MB

7.2 Luau declara, Rust simula. Systems embutidos em Rust (movimento, gravidade, atrito, animação, vida útil, colisão) são configurados por dados. Luau roda só decisões e eventos. Mais rápido e menos código para o agente escrever.
Ponte por buffers: o estado das entidades fica em Rust num buffer contíguo (SoA) que o Luau lê pelo tipo nativo buffer, sem FFI por campo. Comandos de desenho/ação voltam por outro buffer. Posições usam o tipo nativo vector (sem alocação, sem GC).
O agente nunca vê buffers: a API continua simples (ent.pos(e), ent.set_vel(e, vx, vy)) e lê/escreve nos buffers por baixo.
Handles inteiros, sem tabela por entidade.

7.3 B. Luau rápido

A geração de código nativo do Luau (x64/ARM64) costuma dar 1,5x a 2,5x em código intensivo de cálculo. No mlua ela entra pela feature luau-jit, ainda experimental. Marcar só as funções quentes com @native/--!native, medir antes de ligar por padrão, e manter o interpretador como fallback (que já é rápido).
--!strict em todo o código: ajuda o check e o código nativo.
GC com orçamento: passo incremental no fim do frame.
Pools e zero closures/tabelas criadas por frame (lint do check).
Bytecode pré-compilado no engine build; hot reload só em dev.

7.4  Dados e simulação (Rust)

ECS/SoA: arrays contíguos, IDs com geração (u32 índice + geração), sem Box/Rc no hot path.
Zero alocação por frame: bump arena resetada a cada frame, smallvec para listas pequenas.
Colisão: spatial hash com células fixas, broadphase incremental.
Matemática determinística entre SOs: não usar o libm do sistema (sin/cos podem diferir entre Linux e Windows e quebrar replays). Usar a crate libm (Rust puro) ou ponto fixo na física; sem fast-math. Golden hashes no CI nos dois SOs.

7.5 Renderer por software

Framebuffer indexado (u8): 320×180 = ~57KB, cabe no cache. A paleta vira RGBA só no final, por LUT com SIMD.
Blit por linha (memcpy para sprite opaco), máscara em vez de branch por pixel, clipping antes de rasterizar.
Ordenação por camada com counting sort inteiro, atlas único (batch contíguo).
GPU só como apresentador: um quad com textura 320×180 e nearest. Escala inteira, vsync e baixa latência de graça. A rasterização continua na CPU, preservando o determinismo.
Multithread só se medir: tiles disjuntos continuam determinísticos. Padrão = single-thread. Pipeline (update N+1 enquanto renderiza N) é opção.

7.6 Latência e pacing

Timestep fixo com interpolação no render, spin-sleep híbrido.
Input lido no início do frame e apresentado logo após o update.

7.8 Boot, assets e áudio

.pak derivado no build: atlas + metadados binários com mmap e zero parse. A fonte da verdade continua sendo PNG + TOML; o .pak é só artefato.
Carga preguiçosa de assets.
Áudio: thread própria, mixer sem alocação, ring buffer lock-free, samples pré-decodificados.

7.9 Build

PGO além de LTO fat, codegen-units=1, panic=abort.
Duas variantes de SIMD (baseline e AVX2) com detecção em runtime.
Linker mold/lld em dev. Plugins Luau não exigem recompilar a engine.

7.10 Medição

engine profile: ms por sistema, FFI e alocações, em TOON curto (o agente otimiza sozinho com isso):

frame_ms: 1.4
systems[4]{name,ms,ffi,allocs}:
  luau.update,0.62,38,0
  physics,0.31,0,0
  raster,0.38,0,0
  audio,0.05,0,0
  
engine bench: headless determinístico, mesma seed, mesmo resultado.
xtask perf-budget no CI: falha se o fps headless cair mais de 5% ou algum sistema passar do orçamento.


8. agent-friendly

8.1 a engine deve funcionar em ciclos,

1 escreve, 2, ciclo check + run + verificar, um request, 3 observar se erro 4 corrigir

barato e conclusivo.

8.2 Comando /	Função

engine screenshot	/ PNG do frame atual
engine filmstrip --frames 0,10,20,30 /	Uma imagem com N frames lado a lado (movimento e juice em 1 imagem)

engine ascii /	Frame em texto, barato em tokens

engine state /	Entidades nomeadas/tagueadas, posições, valores (TOON tabular)

engine diff a.png b.png	/ oq mudou, com região destacada

engine log /	Eventos (colisao, spawn, morte, som) em TOON tabular; gravados em disco como JSONL

engine record	/ GIF/MP4 curto para o humano

