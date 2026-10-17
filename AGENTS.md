# AGENTS.md — Genflust Engine

Rust core + Luau, pixel art 2D, CLI-first, agent-first. Regras para quem contribui (humano ou agente) no repositório **da engine**. Fonte de verdade: [ENGINE_VISION2.md](ENGINE_VISION2.md) (`§N` = seção); em conflito, o doc vence. Projetos de jogo têm outro `AGENTS.md` (<1k tokens, gerado por `engine new`).

**Status:** planejamento. Itens *(proposta)* não são contrato: validar no jogo de referência (fase 3) antes de congelar. Não inventar API, flag, comando ou formato fora do doc; na dúvida, registrar a pergunta na task em vez de decidir sozinho.

**Teste de toda feature (§1):** o agente consegue usar, verificar e consertar isso só com texto e poucos tokens? Se não, não entra.

---

## 1. Regras duras (violou = rejeitar a mudança)

| # | Regra | Verificação |
|---|---|---|
| R1 | Arquivo ≤300 linhas (alvo 200). Um propósito por módulo, funções pequenas | `xtask arch-check` |
| R2 | Trait ≤5 métodos, uma capacidade. Sem trait "Deus". Trait só em fronteira real de substituição | `arch-check` |
| R3 | Sem globals/singletons. Dependências por construtor. Só `app` conhece impls concretas | `arch-check`, review |
| R4 | Grafo sem ciclos. Núcleo não importa backend nem ferramenta | `arch-check` |
| R5 | Luau decide, Rust executa. Comportamento uniforme em lote = system Rust, não callback Luau | review |
| R6 | Hot path: zero alocação, sem `Box`/`Rc`, nunca 1 chamada FFI por entidade/campo | `profile`, `perf-budget` |
| R7 | Determinismo: timestep fixo, RNG semeado, crate `libm`/ponto fixo, sem fast-math, ordem estável | golden hashes Linux+Windows |
| R8 | Criar/remover entidade, trocar componente/controlador/cena só na barreira do início do próximo tick | testes do mundo |
| R9 | `draw()` e consumidores de observação (Live View, log, MCP) nunca alteram a simulação | hash idêntico com observação on/off |
| R10 | Handle obsoleto nunca é aceito (índice + geração) | teste de contrato |
| R11 | Mensagem/evento de gameplay: fila, entrega no próximo tick, nunca callback síncrono recursivo, nunca descartado em silêncio | teste de contrato |
| R12 | Saída estruturada só pelo encoder (`Formatter`), nunca TOON à mão. Determinística: sem timestamp, ID aleatório, ordem instável | `token-budget` |
| R13 | Silêncio no sucesso. Erro = `code` estável + local + `fix` + `docs`; sem stack trace por padrão | review |
| R14 | Fonte única: schema Rust gera `.d.luau`, docs, validação e campos do editor. Sem segundo cadastro manual | `check` |
| R15 | Cena/preset = TOML, comportamento = Luau, pixels = PNG. CLI e editor: mesmo carregador, validador e TOML (ordem estável) | testes |
| R16 | API estável: não quebrar sem camada de compatibilidade. ~60 funções; adicionar exige remover ou justificar | review, L0 |
| R17 | Agente nunca gera arte final. Usa nome + placeholder, ajusta só metadados TOML | review |
| R18 | Sem fallback silencioso: dependência, módulo, socket ou destino inválido = erro com correção | `check`, `test` |
| R19 | Medir antes de otimizar. Multithread e `luau-jit` só com perfil que justifique, desligados por padrão | `perf-budget` |
| R20 | Mudar contrato do núcleo: justificar + versionar (pré-1.0: minor + migração; ≥1.0: major) | review |
| R21 | Respostas do agente enxutas: direto ao ponto, sem repetir o pedido, sem listas decorativas, sem sumários longos | review |

---

## 2. Arquitetura (§3.3–3.5)

Direção de dependência *(derivada de §3.3/3.4)*: ferramentas → capacidades → núcleo → `api`. Nunca o inverso.

| Camada | Crates (responsabilidades lógicas) |
|---|---|
| Contratos | `api`: só traits/tipos (Renderer, InputSource, AudioSink, AssetLoader, Command, Panel, Fx, EventSink, Formatter). Zero implementação |
| Núcleo | `events` (barramento tipado), `core` (loop, tempo, RNG, cenas leves, ciclo de vida, fases), `ecs` (SoA, IDs com geração, relações, buffers Luau) |
| Capacidades | `gfx`, `input`, `audio`, `physics`, `systems`, `juice`, `script` (mlua/Luau, sandbox), `assets`, `plugins` (host) |
| Ferramentas | `cli`, `fmt-toon`, `fmt-json`, `agent`, `profiler`, `live`, `window`, `mcp`, `editor-*`. Sem lógica de jogo; chamam o mesmo core do CLI |
| Raiz | `app` = composition root. `kits/` = plugins Luau fora do core. `xtask/` = `arch-check`, `token-budget`, `perf-budget` |

- Começar com **módulos internos**; extrair crate só com fronteira de dependência, feature opcional ou reuso real. Não criar todos os crates de uma vez.
- Runtime mínimo: `api events core ecs gfx input script assets cli fmt-toon app`. Headless sem janela.
- Features: `window audio physics animation tilemap juice mcp live editor`. `engine.toml` declara capacidades; `check`/`build` validam disponibilidade. Componente de áudio/física **nunca** ativa módulo indisponível em silêncio.
- Consulta/comando direto → contrato público; notificação → evento. Não transformar consulta simples em cadeia de eventos. Função interna não vira plugin.
- Cada crate testável isolado com fakes. Cada trait tem suíte de contrato rodada em **todas** as impls (renderer software ↔ GPU sem o core notar).
- Plugins: Rust estático (oficiais, custo zero) e Luau em runtime (`plugins/<nome>/plugin.toml`, sandbox, permissão `fs = none|project`). Dylib Rust proibido (ABI frágil); WASM é futuro.
- Manifest: `name`, `version`, `api` (faixa compatível), `requires`, permissões, `l0`. Resolver e validar o grafo antes de executar (versão incompatível, dependência ausente, ciclo → erro com correção). Fixar versões/revisões.
- Falha de plugin: erro com nome do plugin e capacidade afetada. Ferramenta opcional → desativa a ferramenta. Gameplay/dependência obrigatória → pausa a simulação com diagnóstico; processo segue disponível para inspeção.
- L0 (`llms.txt`) lista só plugins **ativos**. Todo plugin traz `l0`, `docs/`, `examples/`, `.d.luau`.

---

## 3. Contratos de runtime (§3.2, 3.7–3.10)

**Tick (ordem fixa, parte do contrato de replay):** 1) aplicar mudanças estruturais e transição de cena pendentes → 2) entregar mensagens/eventos do tick anterior → 3) input → 4) Luau `update(dt)` + controladores ativos (ID estável) → 5) systems Rust habilitados + relações/transformações (batch) → 6) eventos ordenados + estado observável → 7) Luau `draw()` (só gera comandos) → 8) raster → framebuffer → janela | PNG | hash. `load()` conclui a montagem inicial antes do 1º tick.

**Entidade:** ID opaco (`u32` índice + geração), cena de origem, estado de ciclo de vida, nome opcional, tags. Slot reutilizado não revalida handle antigo. ID de runtime nunca vai para arquivo: cenas usam `key` local resolvida em carga.
- Ciclo: criação pendente → ativa → remoção pendente → removida. Remoção marca e bloqueia novos comandos de gameplay; limpeza na próxima barreira, sem invalidar iteração em curso. `ent.new` reserva ID no tick e ativa na barreira; montagem pode usar ID pendente na mesma fila, com validação antes da ativação.
- API pública é `ent`. Sem segunda família `actor`, sem herança. Player/NPC = mesmos componentes e systems; muda só a origem das intenções (input, replay, IA).

**Componentes:** dados opcionais (`Transform`, `Sprite`, `Collider`, `AudioSource`, `Animator`...). Entidade pode ser só lógica. Dependências explícitas (`Sprite`→`Transform`; `Collider`→`Transform` + capacidade física), validadas em `check` **e** em runtime.

**Controlador (1 por entidade no MVP):** callbacks opcionais `load(e)`, `update(e,dt)`, `on_message(e,msg)`, `on_event(e,evt)`, `unload(e)`. Recebe handle validado + contexto público, nunca o armazenamento interno. Roda após o `update` de `main.luau`, por ID estável, antes dos systems Rust.
- `load` 1x ao ativar; `unload` 1x na remoção/substituição; remover cancela inscrições e recursos; falha no `unload` não impede a limpeza do host.
- Estado persistente = dados serializáveis, separado do código; entra em snapshot/replay. Hot reload preserva dados compatíveis; schema mudou → migração declarada ou reinício explícito da cena.
- Composição interna por funções Luau pequenas em ordem explícita. Sem "último a executar vence".

**Mensagens/eventos:** consulta (`ent.pos(e)`) lê sem efeito; mensagem dirigida (`msg.send`) pede ação; evento (`evt.emit`) notifica a cena. Ação local simples usa função pública, não o barramento.
- Entrega no início do próximo tick, ordem estável de emissão e de assinantes. Registro: cena, tick, origem, destino, tipo, sequência.
- Payload: schema, tamanho limitado, serializável. Referência a entidade revalidada na entrega. Destino removido → descarta com diagnóstico estruturado (`test` pode falhar por isso). Referência obrigatória inválida na cena = erro de carga.
- Inscrições pertencem a entidade/controlador/cena e saem automaticamente. Filas pré-alocadas com limites (capacidade, entrega/tick, profundidade de cadeia); estourar = erro reproduzível. Filas pendentes entram em snapshot e hash.

**Relações** (`ent.attach(child,parent,socket,[opts])` *(proposta)*, `ent.detach(child)`): três contratos separados.
- Lógica (`owner`, alvo, item equipado): não move nem destrói. Espacial: um pai, sem ciclos, ordem pai→filho. Ciclo de vida: `detach` (padrão) ou `destroy`.
- `attach` valida IDs, cena, `Transform`, socket. Socket inexistente = erro (nunca `(0,0)`). Ciclo rejeitado antes de alterar. Preserva posição mundial por padrão; alinhar ao socket exige opção. Pai removido com `detach` mantém a última transformação mundial válida; `destroy` remove o filho na mesma barreira. Relação espacial não sobrevive à saída da cena (MVP).
- Filho mantém identidade, controlador e config (espada não herda do Player; Speaker mantém volume). Equipar/soltar = kit sobre essas primitivas, não `player:equip()` no núcleo. Sem sistema de ossos.

**Cena:** TOML (`schema = 1`, `[[entities]]` com `key` única, `tags`, `components.*`, `controller.module`). MVP: uma cena ativa (cena implícita p/ projeto só com `main.luau`); empilhamento/streaming depois. Transição: preparar e validar a próxima antes de trocar; falhou → mantém a atual. Troca na barreira e limpa entidades, ouvintes e filas da anterior. Dados entre cenas só por estado de sessão explícito e serializável, sem handles antigos.

**Presets** (kits, não subclasses): `player` `npc` `static` `speaker` `empty` `relay` `spawner`. Inventário, equipamento e diálogo são dos kits, nunca campos de toda entidade.

---

## 4. Performance (§3.6)

Metas (validar por benchmark, por perfil de build e hardware declarado): boot <100ms · ~1000 entidades <2ms CPU/frame (orçamento 16,6ms @60Hz) · 10k entidades em systems Rust <2ms · FFI <200/frame · alocação no hot path 0 · headless >10k fps · input→tela ≤1 frame · pausa de GC Luau <0,5ms · idle <30MB · binário <5MB.

- **Fronteira Rust↔Luau:** estado em buffer contíguo SoA lido pelo tipo `buffer`; comandos voltam por outro buffer; posições em `vector`. Handles inteiros, sem tabela por entidade. Nomes (asset, cena, componente) resolvidos em carga/check, não no hot path. O agente **nunca vê buffers**: API simples (`ent.pos(e)`, `ent.set_vel(e,vx,vy)`).
- **Dados:** ECS/SoA, bump arena resetada por frame, `smallvec` p/ listas pequenas, spatial hash com células fixas e broadphase incremental. Systems embutidos (movimento, gravidade, atrito, animação, vida útil, colisão) configurados por dados.
- **Render:** framebuffer indexado `u8` (320×180 ≈ 57KB); paleta → RGBA no fim por LUT com SIMD; blit por linha, máscara em vez de branch por pixel, clipping antes de rasterizar, counting sort por camada, atlas único. GPU só apresenta (quad `nearest`, escala inteira); raster fica na CPU. Multithread só medido (tiles disjuntos); padrão single-thread.
- **Luau:** `--!strict` sempre; `@native`/`luau-jit` (experimental) só em função quente e medido, interpretador como fallback; GC incremental no fim do frame; sem closures/tabelas por frame; bytecode pré-compilado no `build`; hot reload só em dev.
- **Tempo:** update fixo 60Hz, render desacoplado com interpolação, spin-sleep híbrido; input lido no início do frame.
- **Assets/áudio:** fonte da verdade PNG+TOML; `.pak` (atlas + metadados binários, mmap) é só artefato do build; carga preguiçosa. Áudio em thread própria, mixer sem alocação, ring buffer lock-free, samples pré-decodificados.
- **Build:** `lto="fat"`, `codegen-units=1`, `opt-level=3`, `panic="abort"`, `strip=true`; PGO e SIMD baseline+AVX2 (detecção em runtime) conforme medição; `mold`/`lld` em dev.
- **Medição:** `engine profile` (ms/sistema, FFI, alocações), `engine bench` (headless, mesma seed = mesmo resultado). `xtask perf-budget` falha se fps headless cair >5% ou sistema estourar orçamento. Testar também em hardware fraco.

---

## 5. Luau e API pública (§2.5, 4.7, 8)

- Forma familiar: `load()` / `update(dt)` / `draw()` (LÖVE), nomes curtos (PICO-8), Lua padrão (tabelas, closures, `:`). Um nome por coisa, sem sinônimos. 320×180 e paleta de 32 cores (arquivo de texto), escala inteira, sem filtro/subpixel.
- Sandbox: sem `io`, `os`, rede. Orçamento de tempo por frame. Erro de script contido na VM: diagnóstico, frame anterior mantido, simulação pausada **no tick que falhou**; retomar de snapshot/tick confirmado ou reiniciar a cena.
- Lints do `check` (cada um com `fix`): alocação no `update`, `dt` nulo, NaN, tabela/closure por frame no `draw`, global acidental.
- L0 (`llms.txt`) ≤3k tokens, 1 linha por função, prefixo imutável (cache de prompt). Bindings geram `.d.luau` e docs da mesma definição Rust.
- Função nova: remover outra ou justificar → L0 + `.d.luau` + `engine docs` + exemplo rodando com `--expect`.

| Módulo | Funções (`[p]` = proposta a validar) |
|---|---|
| `gfx` | `clear(c)` `rect(x,y,w,h,c)` `rectb(x,y,w,h,c)` `spr(id,x,y,[flipx],[flipy])` `text(s,x,y,c)` `cam(x,y)` |
| `input` | `down(k)` `pressed(k)` `axis(neg,pos)` |
| `snd` | `play(name,[vol])` |
| `ent` | `new(tag,[x],[y])→e` `each(tag)→iter` `kill(e)`; `[p]` `pos(e)` `set_vel(e,vx,vy)` `attach` `detach` |
| `scene` `msg` `evt` | `[p]` `scene.load(name)` `msg.send(target,name,payload)` `evt.emit(name,payload)` |
| `phys` | `overlap(a,b)→bool` `sweep(e,dx,dy)` |
| `juice` | `shake(amp,t)` `hitstop(t)` `tween(o,k,to,t,ease)` `burst(x,y,kind,n)` `squash(e,sx,sy)` `fx(name,x,y)` |
| global | `rnd(n)` `rnd_seed(s)` `log(tag,data)` |

---

## 6. CLI, saída e economia de tokens (§4.5, 4.12–4.14, 5)

- Todo comando funciona sem janela. GUI e MCP são wrappers do mesmo core: **CLI é a verdade**. Exit codes significativos; idempotente quando possível.
- Flags globais: `--format toon|json|text` (padrão toon), `--budget N` (padrão 500 tokens; trunca e resume), `--since last` (delta), `--full`. Truncamento com contagem: `+47 itens (--full ou --page 2)`.
- Formatos: TOON = o que o agente **lê** (tabelas, saídas); TOML = o que se **edita** (config, manifests, VFX, metadados, cenas); Luau/flags = o que o agente **escreve**; PNG = pixels; JSONL = logs/gravação append-only (replay final `inputs.toon`: `inputs[N]{frame,key,action}:`). Dado aninhado/não uniforme pode ficar maior em TOON: medir e achatar.
- Erro (≤150 tokens): `code` estável, `file`, `line`, `msg` curto, `fix`, `docs`. Vários = tabela `errors[N]{code,file,line,msg,fix}:`. "Did you mean" para entidade, asset, função e evento.
- `cycle` = check → run headless → asserts → resumo, 1 chamada. Sucesso: `cycle: ok`. Falha: só `failed[N]{test,frame,expected,actual}:` + caminho do screenshot.
- Observar: `screenshot`, `filmstrip` (mosaico, nunca N imagens), `ascii`, `state` (resumo; `--full` detalha; `--filter tag:x`), `log`, `diff`, `record`, `why <pergunta>` (causas ranqueadas com `fix`).
- Verificar: `replay`, `test --expect "player.hp==3 @frame 120"`, `playtest` (crash, softlock, NaN, vazamento de entidades), `invariants.toml` por frame, `snapshot save|restore`, `rewind`.
- Contexto: `map`, `status`, `resume`, `docs`, `examples`, `search`, `api`. Comando novo só com métrica de ganho (menos chamadas ou tokens); senão vira plugin.
- Telemetria local opt-in (nada sai sozinho). Commit por ciclo ok em `engine/auto` (opt-out); snapshots limitados, em pasta ignorada.

| Orçamento (CI `xtask token-budget`) | Alvo |
|---|---|
| `llms.txt` (L0) | ≤3k tokens |
| `cycle` ok / fail | ≤10 / ≤200 |
| erro individual | ≤150 |
| `state` padrão | ≤500 |
| `map` / `resume` | ≤300 / ≤400 |
| `filmstrip` 4 frames | ≈300 (imagem) |

---

## 7. MCP (§4.10)

Wrapper fino do core do CLI; tools orientadas a workflow, **≤8**: `cycle`, `observe` (`view`: screenshot|filmstrip|state|ascii), `why`, `eval`, `docs`, `snapshot`, `art_todo`. Descrição <100 chars; detalhe via `docs`. Toda tool aceita `budget`; truncado devolve `truncated: true` + cursor. `readOnlyHint` em `observe`/`why`/`docs`; destrutivas (`snapshot restore`) marcadas. `eval`/`cycle` filtram no servidor (só o resumo volta). `llms.txt`, `AGENTS.md`, `api` e plugins ativos = *resources*; receitas (`new-game`, `fix-failing-test`, `add-juice`) = *prompts*. Sessão persistente com hot reload. Tool de plugin só aparece com o plugin ativo.

---

## 8. Arte, editores e Live View (§4.8, 6, 7)

- **Arte:** contrato por nome (`gfx.spr("player_idle")`). Nome inexistente → placeholder (caixa colorida + rótulo) e registro em `ART_TODO.md` (`engine art todo`: nome, tamanho, frames, descrição). Metadados em `.toml` ao lado do `.png` (animações, durações, pivôs, hitboxes): agente ajusta timing/hitbox, nunca pixels. `engine assets` lista o que existe. Sprite ASCII só p/ placeholder/teste.
- **Editores** (feature `editor`, binário separado; jogo <5MB): só PNG + TOML, mesmo crate `assets`, hot reload no jogo rodando. Pixel art: lista de ferramentas fechada (§6.1); fora da v1: pincéis complexos, filtros, PSD. VFX é parâmetro (`.toml`), não pixel: `juice.fx("explosion",x,y)`. Editor não é necessário para criar/rodar/testar/buildar. UI: `egui` recomendado no MVP *(decisão em aberto)*.
- **Live View** (plugin `Panel`+`Fx`, desligável, custo ~0 desligado): só assina o barramento `events` (`file_changed`, `reload`, `check_ok/fail`, `entity_spawned`, `state_changed`, `test_pass/fail`, `task_done`...). Todo efeito carrega informação; cores fixas (verde ok, vermelho erro, amarelo mudança, azul novo); ≤400ms; nunca cobre a área do jogo (efeito global só na moldura); som opcional chiptune; `--calm` e temas acessíveis (`.toml`); mesma linguagem visual para todos os eventos.

---

## 9. Testes e CI (§3.4, 4.3)

- Determinismo total: timestep fixo + RNG semeado + inputs gravados = mesma saída. Golden screenshots (hash de framebuffer) e golden hashes Linux e Windows.
- Contratos do mundo (obrigatórios): ID obsoleto, troca de cena, remoção durante update, dependência de componente, entrega ordenada, limpeza de ouvintes, mensagem p/ removido, estouro de fila, attach/detach (independência, associação, separação, remoção do pai, rejeição de ciclo).
- Snapshot completo: cena, componentes, estado dos controladores, relações e políticas, filas pendentes, RNG. Salvar só posição não basta; restaurar reproduz a continuação.
- Observação on/off não muda hash. Testes do jogo em Luau, headless. Exemplos de `engine examples` rodam com `--expect` no CI.
- CI: `xtask arch-check` (linhas, grafo, contratos, métodos por trait), `token-budget`, `perf-budget`, suíte de contrato por trait em todas as impls.
- Task = 1 arquivo (Backlog.md recomendado; verificar se o MCP atual atende), aceite verificável por comando.

---

## 10. Escopo e fases (§9, 11, 13)

Cada fase termina com fluxo executável e verificável. **Jogo de referência na fase 3**, antes de framework amplo de plugins ou editores.

| Fase | Foco |
|---|---|
| 1 | Núcleo: contratos mínimos + `app`, ECS/SoA com IDs geracionais e barreiras, cena implícita/1 TOML, loop fixo, Luau, renderer software, placeholders, `run --headless`, `screenshot`, `check` básico, erros estruturados |
| 2 | Observar/reproduzir: input/janela, `events`, TOON, `state`, `log`, `filmstrip`, `ascii`, replay, baseline `profile`/`bench` |
| 3 | Jogo de referência por agente: `cycle`, `test --expect`, `llms.txt`/AGENTS mínimos, controlador único, msg/evt ordenados, attach/detach, presets mínimos, troca de cena |
| 4 | Contratos/diagnóstico: `check` completo + lints, `why`, schemas, snapshots/rewind, invariantes, golden hashes nos 2 SOs, registry Rust + host Luau versionado |
| 5 | Contexto + Live View MVP: `--budget`/delta, `map`/`status`/`resume`, `token-budget`, MCP MVP, telemetria |
| 6 | `juice`/áudio opcionais, sidecars TOML, `art todo`, kits, `scaffold`, `new --template`, ponte por buffers, GC com orçamento, `perf-budget` |
| 7–10 | Editor de pixel art → editor de VFX + Live View completo → editor de cenas + MCP completo → distribuição (`build`, `.pak`, PGO/SIMD) |

Jogo de referência (fase 3): Player e NPC compartilham movimento; espada independente equipa/solta; coleta abre porta via `relay`; troca de cena limpa recursos; Speaker opcional acompanha o Player.

**Fora do MVP (só com uso real demonstrado):** múltiplos controladores por entidade, empilhamento/streaming de cenas, sistema de ossos, plugins nativos dinâmicos, WASM, networking, backend GPU de raster, editor de cenas antes da fase 9, tilemaps/state machines/diálogos no editor.

---

## 11. Convenções de código *(derivadas; não vêm do doc — ajustar se discordar)*

- Sem `unwrap`/`expect`/`panic!` em caminho alcançável por script, asset, cena ou CLI (`panic="abort"` mata o processo). Retornar `Result` com erro estruturado (`code`, `fix`).
- `unsafe` só em fronteira de buffer/SIMD, módulo pequeno e isolado, com `// SAFETY:` e teste.
- Simulação e saída observável iteram em ordem estável (`Vec`, índice, `BTreeMap`); nunca `HashMap` iterado.
- Newtypes para `EntityId`, handle de asset, tick e frame; sem inteiros soltos nas APIs.
- Comentário explica o **porquê** (decisão, contrato, `§`), nunca repete o código.
- `cargo fmt` e `clippy -D warnings` limpos. Teste junto do módulo com fakes; teste de contrato fica na suíte da trait.

---

## 12. Tentações comuns → correção

| Tentação | Correto |
|---|---|
| NPC herda de Player; família `actor`; herança | Mesma entidade `ent`, componentes compartilhados, controlador diferente (§3.7–3.8) |
| "Tudo é plugin" / tudo por evento / nunca tocar o core | Plugin só em fronteira de capacidade; consulta por contrato público; contrato do núcleo evolui com versão (§3.4) |
| Vários controladores por entidade | 1 por entidade; compor funções Luau em ordem explícita |
| Callback Luau por partícula/projétil | System Rust ou operação em batch |
| Efeito cosmético como entidade | Sistema de partículas; entidade só com identidade ou interação de gameplay |
| Evento síncrono recursivo | Fila + entrega no próximo tick |
| Socket inválido reposiciona em `(0,0)` | Erro com correção |
| Snapshot só com posição | Cena, componentes, controladores, relações, filas, RNG |
| Continuar tick parcial após falha/hot reload | Retomar de snapshot/tick confirmado ou reiniciar a cena |
| Closure, ponteiro, objeto de VM ou ID de runtime em arquivo | Dados serializáveis + `key` local resolvida em carga |
| Inventário/equipamento/diálogo no núcleo | Kit sobre `attach`/`detach` + regras |
| Editor com formato ou cadastro de propriedades próprio | Mesmo TOML, schema, carregador e validador do CLI |
| Comando de CLI "útil" sem métrica | Só com ganho medido (chamadas/tokens); senão plugin |
| Doc ou exemplo sem teste | Exemplo roda com `--expect` no CI |
| Sinônimo na API | Um nome por coisa |
| Trait/crate para tudo | Módulo interno primeiro; trait/crate só com fronteira real |
| Otimizar por intuição | `engine profile` aponta; `perf-budget` guarda |

---

## 13. Antes de concluir

**Fluxo:** ler a task + `§` citada → conferir R1–R20 → menor mudança que cumpre o aceite → testes → `cargo xtask arch-check` (+ budgets quando aplicável) → checklist.

- [ ] Nenhum arquivo >300 linhas; `arch-check` limpo (linhas, ciclos, métodos por trait).
- [ ] Teste cobre a mudança (unitário com fakes; contrato quando mexe em trait ou no mundo).
- [ ] Mesma seed → mesmos hashes/estado; observação on/off idêntica.
- [ ] Sem alocação nova no hot path; `perf-budget` e `token-budget` dentro do orçamento.
- [ ] Erros novos com `code` estável, `fix` e `docs`.
- [ ] API nova/alterada: L0, `.d.luau`, docs e exemplo com `--expect` atualizados; contrato do núcleo → versão + migração.
- [ ] Itens *(proposta)* tratados como proposta; nada inventado fora do doc.
- [ ] Critério de aceite da task verificável por comando.

---

## 14. Ambiente (Windows e Linux)

Compila nos dois; **o CI é o árbitro** (PR só entra com os dois verdes).

- Toolchain fixa em `rust-toolchain.toml` (1.99.1 + rustfmt + clippy); **`Cargo.lock` versionado**. Só `cargo` e `cargo xtask` — sem `.sh`/`.ps1`.
- LF em tudo: `.gitattributes` (`* text=auto eol=lf`, png/pak binary) + `.editorconfig`. Windows: `git config core.autocrlf false`. CRLF quebra diff de TOML e hash dos goldens.
- Asset/cena em `lower_snake_case`; `check` rejeita nomes que diferem só por caixa. Caminhos sempre com `/`.
- Determinismo (R7): `libm`, sem fast-math, sem `-C target-cpu=native`; golden hashes batem nos dois SOs.
- mold/lld só em `[target.x86_64-unknown-linux-gnu]` (opcional). Hot reload: arquivo temporário + rename, debounce no watcher.
- Windows: target MSVC + VS Build Tools (C++) (mlua = Luau em C++); PowerShell 7 (o 5.1 grava `>` em UTF-16); saída por `--out`, nunca redirecionamento; WSL2 só p/ reproduzir falhas Linux (clone em `~/`, não `/mnt/c`).
- Linux: `build-essential` + `pkg-config`; X11/Wayland/ALSA só com features `window`/`audio`; mold opcional.
- CI: matriz ubuntu+windows → `fmt --check`, `clippy -D warnings`, `test` (goldens), `arch-check`, `token-budget`. `perf-budget`: manual, máquina self-hosted declarada (`perf/baselines.toml`).
- IA local (RTK, `.agents/`) fora do git; quem não usa IA não é afetado. Setup completo: `docs/ambiente.md`.
