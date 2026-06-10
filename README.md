# ternary-mud

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![SuperInstance](https://img.shields.io/badge/part%20of-SuperInstance-purple.svg)](https://github.com/SuperInstance)

Ternary algebra MUD room model with Hodge decomposition for algebraic lostness detection.

## What It Does

`ternary-mud` models a dungeon as a ternary algebra where room connections take values in {-1, 0, +1} (Wall, Threshold, Portal). It solves the discrete Dirichlet problem to compute room atmosphere, applies Hodge decomposition to player traversal flows, and produces a `LostnessReport` — an algebraic measure of how lost a player is, broken into gradient (on-path), harmonic (circling), and curl (dead-end) components.

The conservation law **γ + η = C** appears here as: a player's total traversal energy decomposes into productive gradient energy (γ) and wasted dead-end energy (η), bounded by a constant C determined by dungeon topology.

## Architecture

```
┌─────────────────────────────────────────────┐
│                  lib.rs                      │
│         (re-exports all public types)        │
├──────────┬──────────┬───────────┬───────────┤
│ Ternary  │  Room    │  Dungeon  │ Atmosphere│
│ {-1,0,+1}│ (id,     │ (adjacency│ (Dirichlet│
│ Z₃ group │  exits,  │  matrix,  │  solver,  │
│ compose, │  atmos-  │  Laplacian│  Laplacian│
│ inverse) │  phere)  │  genus)   │  verify)  │
├──────────┴──────────┴───────────┴───────────┤
│          hodge_navigation.rs                 │
│  ┌─────────────┐  ┌──────────────────────┐  │
│  │  Hodge      │  │  Traversal           │  │
│  │  Decompose  │  │  Decompose           │  │
│  │ (gradient + │  │ (+ dead-end/curl     │  │
│  │  harmonic)  │  │  detection)          │  │
│  └─────────────┘  └──────────────────────┘  │
├─────────────────────────────────────────────┤
│            lostness.rs                       │
│  LostnessReport::analyze()                  │
│  → score ∈ [0,1], component, suggested_exit│
└─────────────────────────────────────────────┘
```

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
ternary-mud = { git = "https://github.com/SuperInstance/ternary-mud" }
```

Or clone and build locally:

```bash
git clone https://github.com/SuperInstance/ternary-mud.git
cd ternary-mud
cargo build
```

## Usage

### Build a dungeon and solve atmosphere

```rust
use ternary_mud::*;

// Build a linear 5-room dungeon: 0--1--2--3--4
let dungeon = Dungeon::linear(5);

// Set boundary conditions: room 0 = 0.0, room 4 = 4.0
let atmosphere = solve_dirichlet(&dungeon, &[(0, 0.0), (4, 4.0)]);
// Interior rooms: [0.0, 1.0, 2.0, 3.0, 4.0]
```

### Add portals and measure topology

```rust
let mut d = Dungeon::linear(5);
assert_eq!(d.genus(), 0); // tree, no cycles

// Add a wormhole shortcut: room 0 ↔ room 4
d.connect_bidir(0, 4, Ternary::Portal);
assert_eq!(d.genus(), 1); // one independent cycle
assert_eq!(d.portal_count(), 1);
```

### Detect player lostness

```rust
let d = Dungeon::linear(5);

// Player stuck visiting dead-end rooms 0 and 4 repeatedly
let visits = vec![10.0, 0.0, 0.0, 0.0, 10.0];
let report = LostnessReport::analyze(&d, &visits, /* current */ 4, /* goal */ 0);

assert_eq!(report.component, LostnessComponent::DeadEnd);
assert!(report.score > 0.5);
assert_eq!(report.suggested_exit, Some(3)); // go toward goal
```

### Hodge decomposition of traversal flow

```rust
let d = Dungeon::ring(4);
let visits = vec![1.0, 1.0, 1.0, 1.0]; // uniform visits
let decomp = hodge_decompose(&d, &visits);

// Uniform visits on a ring → all energy is harmonic (circling)
assert!(decomp.gradient_norm() < 1e-10);
assert!(decomp.harmonic_norm() > 0.1);
```

## API Reference

### `Ternary` — Core ternary value `{-1, 0, +1}`

| Method | Signature | Description |
|--------|-----------|-------------|
| `value` | `fn value(self) -> i8` | Numeric: -1, 0, or +1 |
| `compose` | `fn compose(self, other: Self) -> Self` | Z₃ addition (mod 3) |
| `inverse` | `fn inverse(self) -> Self` | Z₃ inverse: Wall ↔ Portal |
| `negate` | `fn negate(self) -> Self` | Sign flip |
| `parallel` | `fn parallel(self, other: Self) -> Self` | Max-like parallel composition |

### `Dungeon` — Graph of rooms with ternary adjacency

| Method | Signature | Description |
|--------|-----------|-------------|
| `add_room` | `fn add_room(&mut self, room: Room) -> usize` | Add room, return index |
| `connect` | `fn connect(&mut self, from: usize, to: usize, conn: Ternary)` | Directed connection |
| `connect_bidir` | `fn connect_bidir(&mut self, a: usize, b: usize, conn: Ternary)` | Bidirectional |
| `genus` | `fn genus(&self) -> usize` | Cyclomatic number: \|E\| - \|V\| + \|C\| |
| `shortest_path` | `fn shortest_path(&self, start: usize, end: usize) -> Option<Vec<usize>>` | BFS path |
| `laplacian` | `fn laplacian(&self) -> Vec<Vec<f64>>` | Graph Laplacian L = D - A |
| `linear` / `ring` / `tree` | `fn linear(n) -> Self` etc. | Builder constructors |

### `LostnessReport` — Algebraic lostness analysis

```rust
pub fn analyze(
    dungeon: &Dungeon,
    visits: &[f64],
    current_room: usize,
    goal_room: usize,
) -> LostnessReport
```

Returns a report with:
- `score: f64` — 0.0 (on path) to 1.0 (maximally lost)
- `component: LostnessComponent` — `OnPath`, `Circling`, or `DeadEnd`
- `suggested_exit: Option<usize>` — next room toward the goal

### `solve_dirichlet` — Atmosphere solver

```rust
pub fn solve_dirichlet(dungeon: &Dungeon, boundary: &[(usize, f64)]) -> Vec<f64>
```

Solves Δf(v) = 0 at interior rooms with fixed boundary values. Uses Gaussian elimination with partial pivoting — no external dependencies.

## Related Crates (SuperInstance Ecosystem)

- **ternary-cell** — Cellular automata on ternary grids, uses ternary-mud's room model
- **ternary-lattice** — Low-level ternary lattice operations
- **ternary-navigation** — Pathfinding with ternary weights
- **ternary-room** — Room/channel model for fleet coordination
- **ternary-topology** — Topological invariants (Betti numbers, fundamental group)
- **ternary-consensus** — Distributed agreement via ternary voting
- **symplectic-fleet** — Fleet dynamics as symplectic manifold, conservation laws
