# ternary-mud

Ternary algebra MUD room model with Hodge decomposition for algebraic lostness detection.

[![Crates.io](https://img.shields.io/crates/v/ternary-mud.svg)](https://crates.io/crates/ternary-mud)
[![docs.rs](https://docs.rs/ternary-mud/badge.svg)](https://docs.rs/ternary-mud)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                          lib.rs                             │
│  Re-exports: Ternary, Room, Dungeon,                        │
│              solve_dirichlet, discrete_laplacian,           │
│              NavigationDecomposition, hodge_decompose,      │
│              traversal_decompose, LostnessReport            │
└─────────────────────────────────────────────────────────────┘
     │           │           │           │           │
     ▼           ▼           ▼           ▼           ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ ternary_ │  │   room   │  │  dungeon │  │ atmosphere│  │hodge_nav.│
│ lattice  │  │          │  │          │  │          │  │          │
│ (Z₃)     │  │(Context) │  │(Category)│  │(Harmonic)│  │(Decomp.) │
└──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘
                                                            │
                                                            ▼
                                                      ┌──────────┐
                                                      │ lostness │
                                                      │(Metric)  │
                                                      └──────────┘
```

The crate models a Multi-User Dungeon (MUD) as a **category** where objects are rooms and morphisms are ternary-valued doors. It combines three mathematical structures:

1. **Balanced ternary algebra** (`Z₃`) — Room connections are `Wall (-1)`, `Threshold (0)`, or `Portal (+1)`, forming an abelian group under modulo-3 addition.
2. **Discrete Hodge theory** — Navigation flows decompose into gradient (productive movement), harmonic (circling), and curl (dead-end stuckness).
3. **Dirichlet boundary value problems** — Atmosphere (mood/tension) propagates through the dungeon as a harmonic function.

### Key Design Decisions

- **Ternary composition** — `Portal + Portal = Wall`, `Wall + Portal = Threshold`, etc. This is not decoration; it is genuine `Z₃` arithmetic with identity, inverses, and associativity.
- **Portals create topology** — A `Portal` edge adds a cycle, increasing genus. This changes the Hodge decomposition and creates harmonic (circling) behavior.
- **Lostness is algebraic** — Using Hodge decomposition on visit-count vectors, lostness = `curl / (gradient + curl)`. A high score means the player is stuck in dead ends.
- **Atmosphere is harmonic** — Solves `Δf = 0` on interior rooms with fixed boundary values. No external linear algebra dependencies.

---

## Quick Start

```rust
use ternary_mud::{Ternary, Room, Dungeon};
use ternary_mud::{solve_dirichlet, discrete_laplacian};
use ternary_mud::{hodge_decompose, traversal_decompose, NavigationDecomposition};
use ternary_mud::{LostnessReport, LostnessComponent};

// 1. Build a dungeon
let mut dungeon = Dungeon::new();
let entrance = dungeon.add_room(Room::new("entrance"));
let corridor = dungeon.add_room(Room::new("corridor"));
let treasury = dungeon.add_room(Room::new("treasury"));
let trap = dungeon.add_room(Room::new("trap"));

dungeon.connect_bidir(entrance, corridor, Ternary::Portal);
dungeon.connect_bidir(corridor, treasury, Ternary::Portal);
dungeon.connect_bidir(corridor, trap, Ternary::Wall);
dungeon.connect_bidir(treasury, trap, Ternary::Threshold);

assert!(dungeon.is_connected());
assert_eq!(dungeon.genus(), 0); // tree-like topology

// 2. Compute atmosphere via Dirichlet problem
let boundary = vec![(entrance, 1.0), (trap, 0.0)];
let atmosphere = solve_dirichlet(&dungeon, &boundary);
println!("Atmosphere at treasury: {:.3}", atmosphere[treasury]);

// 3. Verify Laplacian is zero on interior
let lap = discrete_laplacian(&dungeon, &atmosphere);
for i in 1..3 {
    assert!(lap[i].abs() < 1e-6, "Room {} is not harmonic", i);
}

// 4. Hodge decomposition of navigation flow
let visits = vec![1.0, 3.0, 5.0, 2.0];
let nav = hodge_decompose(&dungeon, &visits);
println!("Gradient norm: {:.3}", nav.gradient_norm());
println!("Harmonic norm: {:.3}", nav.harmonic_norm());

// 5. Lostness detection
let report = LostnessReport::analyze(&dungeon, &visits, entrance, treasury);
println!("Lostness score: {:.2}", report.score);
match report.component {
    LostnessComponent::OnPath => println!("Player is on the right path"),
    LostnessComponent::Circling => println!("Player is circling — check for portals"),
    LostnessComponent::DeadEnd => println!("Player is stuck in a dead end"),
}
if let Some(exit) = report.suggested_exit {
    println!("Suggest moving toward room {}", exit);
}

// 6. Add a portal to create non-trivial topology
let shortcut = dungeon.add_room(Room::new("shortcut"));
dungeon.connect_bidir(entrance, shortcut, Ternary::Portal);
dungeon.connect_bidir(shortcut, treasury, Ternary::Portal);
assert_eq!(dungeon.genus(), 1); // now has a cycle
```

---

## API Reference

### Ternary Algebra

| Type | Description |
|------|-------------|
| `Ternary::Wall` | `-1` — impassable, dangerous, blocked |
| `Ternary::Threshold` | `0` — neutral doorway, identity element |
| `Ternary::Portal` | `+1` — wormhole, teleport, non-trivial topology |

| Function | Description |
|----------|-------------|
| `Ternary::value()` | Numeric value: `-1`, `0`, or `+1` |
| `Ternary::from_value(v)` | Clamp `i8` to nearest ternary |
| `Ternary::compose(other)` | `Z₃` addition (mod 3) |
| `Ternary::inverse()` | Group inverse: `Wall ↔ Portal` |
| `Ternary::negate()` | Sign flip |
| `Ternary::parallel(other)` | Max-like composition for parallel paths |
| `std::ops::Add` | Alias for `compose` |
| `std::ops::Mul` | Sign multiplication |

### Room & Dungeon

| Function | Description |
|----------|-------------|
| `Room::new(id)` | Create a room |
| `Room::with_exit(name, target, conn)` | Builder: add named exit |
| `Room::with_atmosphere(value)` | Builder: set atmosphere |
| `Room::with_content(item)` | Builder: add item |
| `Room::connection_to(target)` | Lookup connection type to target |
| `Dungeon::new()` | Empty dungeon |
| `Dungeon::add_room(room)` | Add room, return index |
| `Dungeon::connect(from, to, conn)` | Directed connection |
| `Dungeon::connect_bidir(a, b, conn)` | Bidirectional connection |
| `Dungeon::is_connected()` | All rooms reachable |
| `Dungeon::connected_components()` | BFS component IDs |
| `Dungeon::genus()` | Cyclomatic number `|E| - |V| + |C|` |
| `Dungeon::portal_count()` | Number of portal edges |
| `Dungeon::shortest_path(start, end)` | BFS shortest path |
| `Dungeon::laplacian()` | Graph Laplacian `L = D - A` |
| `Dungeon::incidence_matrix()` | Directed incidence matrix for Hodge theory |
| `Dungeon::linear(n)` | Builder: chain of `n` rooms |
| `Dungeon::ring(n)` | Builder: cycle of `n` rooms |
| `Dungeon::tree(depth)` | Builder: binary tree |

### Atmosphere & Harmonic Functions

| Function | Description |
|----------|-------------|
| `solve_dirichlet(dungeon, boundary)` | Solve `Δf = 0` with fixed boundary values |
| `discrete_laplacian(dungeon, values)` | Compute `L · values` at each room |
| `verify_laplacian(dungeon, values, boundary)` | Verify interior points are harmonic |

### Hodge Navigation

| Function | Description |
|----------|-------------|
| `hodge_decompose(dungeon, flow)` | Decompose room flow into gradient + harmonic + curl |
| `traversal_decompose(dungeon, visits)` | Extend Hodge with dead-end detection |
| `NavigationDecomposition::gradient_norm()` | L2 norm of gradient component |
| `NavigationDecomposition::harmonic_norm()` | L2 norm of harmonic component |
| `NavigationDecomposition::curl_norm()` | L2 norm of curl component |

### Lostness Detection

| Function | Description |
|----------|-------------|
| `LostnessReport::analyze(dungeon, visits, current, goal)` | Compute lostness score and suggestion |

| Field | Description |
|-------|-------------|
| `LostnessReport::score` | `0.0` = on path, `1.0` = maximally lost |
| `LostnessReport::component` | `OnPath / Circling / DeadEnd` |
| `LostnessReport::suggested_exit` | Next room toward goal (BFS) |

---

## Integration Notes

### With Game Engines

Use `LostnessReport` to adapt difficulty or provide hints:

```rust
let report = LostnessReport::analyze(&dungeon, &visit_counts, player_room, quest_goal);

match report.component {
    LostnessComponent::DeadEnd => {
        ui.show_hint(format!("Try heading toward room {}", report.suggested_exit.unwrap()));
    }
    LostnessComponent::Circling => {
        ui.show_hint("You've been here before. Look for a new exit.".into());
    }
    LostnessComponent::OnPath => {
        // No hint needed
    }
}
```

### With Procedural Content Generation

Validate generated dungeons before presenting them to players:

```rust
fn validate_dungeon(dungeon: &Dungeon) -> Result<(), String> {
    if !dungeon.is_connected() {
        return Err("Dungeon has unreachable rooms".into());
    }
    if dungeon.genus() > 3 {
        return Err("Too many cycles — players will get lost".into());
    }
    if dungeon.portal_count() == 0 {
        return Err("No portals — dungeon is too linear".into());
    }
    Ok(())
}
```

### With Player Analytics

Track visit patterns and detect stuck players server-side:

```rust
// In your game server's tick loop
for (player_id, visits) in &player_visit_logs {
    let report = LostnessReport::analyze(&dungeon, visits, player_room, player_goal);
    if report.score > 0.7 {
        analytics.record_event("player_stuck", player_id, report.score);
    }
}
```

### With Atmosphere-Driven Narrative

Use the Dirichlet solution to drive dynamic tension:

```rust
let tension = solve_dirichlet(&dungeon, &[(boss_room, 1.0), (entrance, 0.0)]);
for (i, room) in dungeon.rooms.iter().enumerate() {
    room.atmosphere = tension[i];
    if tension[i] > 0.8 {
        spawn_enemy(room);
    } else if tension[i] < 0.2 {
        spawn_loot(room);
    }
}
```

### With Topology Visualization

Export dungeon structure for graph visualization:

```rust
let components = dungeon.connected_components();
let genus = dungeon.genus();
let lap = dungeon.laplacian();

let export = serde_json::json!({
    "rooms": dungeon.rooms,
    "adjacency": dungeon.adjacency,
    "components": components,
    "genus": genus,
    "laplacian": lap,
});
// Send to D3.js / Cytoscape / Graphviz renderer
```

---

## Module Map

| Module | Description | Key Types |
|--------|-------------|-----------|
| `ternary_lattice` | Balanced ternary `Z₃` algebra | `Ternary` |
| `room` | Room context container | `Room` |
| `dungeon` | Dungeon as category/graph | `Dungeon` |
| `atmosphere` | Dirichlet solver + discrete Laplacian | `solve_dirichlet`, `discrete_laplacian` |
| `hodge_navigation` | Hodge decomposition on room graphs | `NavigationDecomposition`, `hodge_decompose` |
| `lostness` | Algebraic lostness metric | `LostnessReport`, `LostnessComponent` |

---

## Testing

```bash
cargo test   # 37 unit tests across all modules
cargo test ternary_lattice::tests::ternary_associativity
cargo test dungeon::tests::dungeon_connectivity_ring
cargo test dungeon::tests::portal_creates_nontrivial_topology
cargo test atmosphere::tests::atmosphere_linear_interpolation
cargo test hodge_navigation::tests::hodge_decomposition_preserves_flow
cargo test lostness::tests::dead_end_player_detected
cargo test lostness::tests::suggested_exit_points_toward_goal
```

---

## License

MIT
