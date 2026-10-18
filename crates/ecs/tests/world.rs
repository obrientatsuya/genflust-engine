//! Contratos do mundo (ENGINE_VISION2 §4.3): IDs geracionais, barreiras,
//! dependências de componentes e chaves únicas. Tudo por API pública.

use ecs::{ComponentError, Lifecycle, Sprite, Transform, World};

#[test]
fn spawn_activates_at_barrier() {
    let mut w = World::new();
    let e = w.spawn("player", Some(1.0), Some(2.0));
    assert_eq!(w.lifecycle(e), Ok(Lifecycle::Pending));
    w.barrier();
    assert_eq!(w.lifecycle(e), Ok(Lifecycle::Active));
    assert_eq!(w.by_tag("player"), vec![e]);
}

#[test]
fn kill_at_barrier_frees_slot_and_invalidates_handle() {
    let mut w = World::new();
    let e = w.spawn("bullet", None, None);
    w.barrier();
    w.kill(e).unwrap();
    assert_eq!(w.lifecycle(e), Ok(Lifecycle::PendingRemoval));
    w.barrier();
    assert_eq!(w.lifecycle(e), Err(ComponentError::Stale));
}

#[test]
fn reused_slot_never_revalidates_old_handle() {
    let mut w = World::new();
    let e1 = w.spawn("a", None, None);
    w.barrier();
    w.kill(e1).unwrap();
    w.barrier();
    let e2 = w.spawn("b", None, None);
    w.barrier();
    assert_eq!(e1.index(), e2.index());
    assert_ne!(e1, e2);
    assert!(w.kill(e1).is_err());
    assert!(w.transform(e1).is_err());
}

#[test]
fn sprite_requires_transform() {
    let mut w = World::new();
    let e = w.spawn("s", None, None);
    w.barrier();
    assert_eq!(
        w.add_sprite(e, Sprite { asset: "x".into() }),
        Err(ComponentError::Dependency {
            component: "Sprite",
            required: "Transform"
        })
    );
    w.add_transform(e, Transform::default()).unwrap();
    assert!(w.add_sprite(e, Sprite { asset: "x".into() }).is_ok());
}

#[test]
fn pending_spawn_can_be_mounted_in_same_tick() {
    let mut w = World::new();
    let e = w.spawn("npc", Some(3.0), None);
    // Montagem no mesmo tick usa o ID pendente.
    assert!(w.add_transform(e, Transform { x: 9.0, y: 9.0 }).is_ok());
    w.barrier();
    assert_eq!(w.transform(e).unwrap().x, 9.0);
}

#[test]
fn duplicate_key_rejected() {
    let mut w = World::new();
    let a = w.spawn("a", None, None);
    let b = w.spawn("b", None, None);
    w.barrier();
    w.set_name(a, "porta").unwrap();
    assert_eq!(
        w.set_name(b, "porta"),
        Err(ComponentError::DuplicateKey("porta".into()))
    );
}

#[test]
fn kill_pending_cancels_spawn_at_barrier() {
    let mut w = World::new();
    let e = w.spawn("spark", None, None);
    // Matar ainda pendente cancela a criação na mesma fila de barreira.
    w.kill(e).unwrap();
    w.barrier();
    assert_eq!(w.lifecycle(e), Err(ComponentError::Stale));
    assert!(w.by_tag("spark").is_empty());
}

#[test]
fn invalid_batch_leaves_no_partial_state() {
    let mut w = World::new();
    let alive = w.spawn("alive", None, None);
    w.barrier();

    // Lote inválido: handle obsoleto + componente sem dependência.
    let stale = {
        let tmp = w.spawn("tmp", None, None);
        w.barrier();
        w.kill(tmp).unwrap();
        w.barrier();
        tmp
    };
    assert!(w.kill(stale).is_err(), "handle obsoleto rejeitado");
    assert!(
        w.add_sprite(stale, Sprite { asset: "x".into() }).is_err(),
        "obsoleto nunca registra componente"
    );
    assert_eq!(
        w.add_sprite(alive, Sprite { asset: "x".into() }),
        Err(ComponentError::Dependency {
            component: "Sprite",
            required: "Transform"
        })
    );
    // Montar o Transform e tentar de novo funciona: a falha não corrompeu nada.
    w.add_transform(alive, Transform::default()).unwrap();
    w.add_sprite(alive, Sprite { asset: "x".into() }).unwrap();
    w.barrier();
    assert_eq!(w.sprite(alive).unwrap().asset, "x");
}

#[test]
fn soa_contiguity_and_stable_iteration() {
    let mut w = World::new();
    let a = w.spawn("a", Some(1.0), Some(1.0));
    let b = w.spawn("b", None, None);
    let c = w.spawn("c", Some(3.0), Some(3.0));
    w.barrier();
    assert_eq!(
        w.component_counts().0,
        2,
        "coluna contígua tem só presentes"
    );
    assert!(w.transform(b).is_err());

    // remover o do meio não bagunça o mapeamento
    w.kill(a).unwrap();
    w.barrier();
    assert_eq!(w.component_counts().0, 1);
    assert_eq!(w.transform(c).unwrap(), Transform { x: 3.0, y: 3.0 });
    // iteração estável por slot: b antes de c
    let slots: Vec<u32> = w.active().iter().map(|e| e.index()).collect();
    assert_eq!(slots, vec![b.index(), c.index()]);
}

#[test]
fn snapshot_is_canonical_and_excludes_transients() {
    // Mesmo estado final montado em ordens diferentes → mesma serialização.
    let build_a = || {
        let mut w = World::new();
        let a = w.spawn("a", Some(1.0), Some(2.0));
        w.add_sprite(
            a,
            Sprite {
                asset: "player_idle".into(),
            },
        )
        .unwrap();
        let b = w.spawn("b", Some(3.0), Some(4.0));
        w.set_velocity(b, 1.5, 0.0).unwrap();
        w.barrier();
        w.set_name(a, "a").unwrap();
        w.set_name(b, "b").unwrap();
        w
    };
    let build_b = || {
        let mut w = World::new();
        let b = w.spawn("b", None, None);
        w.add_transform(b, Transform { x: 3.0, y: 4.0 }).unwrap();
        w.set_velocity(b, 1.5, 0.0).unwrap();
        let a = w.spawn("a", None, None);
        w.add_transform(a, Transform { x: 1.0, y: 2.0 }).unwrap();
        w.add_sprite(
            a,
            Sprite {
                asset: "player_idle".into(),
            },
        )
        .unwrap();
        w.barrier();
        w.set_name(a, "a").unwrap();
        w.set_name(b, "b").unwrap();
        w
    };

    let mut wa = build_a();
    let wb = build_b();
    assert_eq!(
        wa.snapshot().to_toml(),
        wb.snapshot().to_toml(),
        "ordem canônica independente da ordem de consulta"
    );

    // transitório (remoção pendente) fica fora do snapshot
    let c = wa.spawn("c", Some(9.0), Some(9.0));
    wa.barrier();
    wa.set_name(c, "c").unwrap();
    wa.kill(c).unwrap();
    let snap = wa.snapshot();
    assert!(
        !snap.entities.iter().any(|e| e.key == "c"),
        "entidade em remoção pendente não serializa"
    );
}

#[test]
fn snapshot_roundtrips_through_toml() {
    let mut w = World::new();
    let a = w.spawn("a", Some(1.0), Some(2.0));
    w.add_sprite(
        a,
        Sprite {
            asset: "coin".into(),
        },
    )
    .unwrap();
    w.set_velocity(a, 0.5, -0.5).unwrap();
    w.barrier();
    w.set_name(a, "a").unwrap();

    let snap = w.snapshot();
    let text = snap.to_toml();
    let back = ecs::Snapshot::from_toml(&text).unwrap();
    assert_eq!(snap, back, "roundtrip preserva o estado persistente");
    assert!(text.contains("coin"));
}
