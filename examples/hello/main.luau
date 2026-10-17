--!strict
-- Olá, Genflust: cena TOML + entidades + draw determinístico (fase 1).
-- Callbacks são globais por contrato (LÖVE): load() update(dt) draw().

local px = 0.0

function load()
	-- A cena já foi montada pelo Rust antes do primeiro tick.
	local e = ent.new("spark", 10, 10)
	ent.kill(e)
end

function update(dt)
	px = px + (rnd(3) - 1.0) -- passeio aleatório determinístico (seed fixa)
end

function draw()
	gfx.clear(0)
	gfx.rectb(0, 0, 320, 180, 7)
	gfx.spr("player_idle", 48, 48)
	gfx.text("player", 48, 62, 7)
	gfx.spr("coin", 160, 90)
	gfx.rect(math.floor(px), 10, 8, 8, 8)
	gfx.text("hello genflust", 8, 168, 7)
end
