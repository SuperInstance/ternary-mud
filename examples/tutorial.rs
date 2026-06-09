//! Tutorial: ternary-mud — Balanced ternary MUD game engine

use ternary_mud::{
    Ternary, Room, Dungeon,
    solve_dirichlet, discrete_laplacian,
    hodge_decompose,
    LostnessReport,
};

fn main() {
    println!("=== Ternary MUD Tutorial ===\n");

    // Part 1: Ternary connection types
    println!("Part 1: Ternary values");
    println!("  Wall (-1):     blocked / dangerous (value={})", Ternary::Wall.value());
    println!("  Threshold (0): neutral / unexplored (value={})", Ternary::Threshold.value());
    println!("  Portal (+1):   open / safe (value={})", Ternary::Portal.value());
    println!();

    // Part 2: Build a dungeon
    println!("Part 2: Building a dungeon");
    let mut dungeon = Dungeon::new();
    let entrance = dungeon.add_room(Room::new("entrance"));
    let corridor = dungeon.add_room(Room::new("corridor"));
    let treasury = dungeon.add_room(Room::new("treasury"));
    let trap = dungeon.add_room(Room::new("trap"));

    dungeon.connect_bidir(entrance, corridor, Ternary::Portal);
    dungeon.connect_bidir(corridor, treasury, Ternary::Portal);
    dungeon.connect_bidir(corridor, trap, Ternary::Wall);
    dungeon.connect_bidir(treasury, trap, Ternary::Threshold);

    println!("  {} rooms, connected: {}", dungeon.len(), dungeon.is_connected());
    println!();

    // Part 3: Atmosphere via Dirichlet
    println!("Part 3: Atmosphere (Dirichlet boundary)");
    let boundary: Vec<(usize, f64)> = vec![(entrance, 1.0), (trap, 0.0)];
    let atmosphere = solve_dirichlet(&dungeon, &boundary);
    for (i, val) in atmosphere.iter().enumerate() {
        println!("  Room {}: atmosphere = {:.3}", i, val);
    }
    println!();

    // Part 4: Laplacian
    println!("Part 4: Discrete Laplacian");
    let lap = discrete_laplacian(&dungeon, &atmosphere);
    println!("  Laplacian: {:?}", lap.iter().map(|v| format!("{:.4}", v)).collect::<Vec<_>>());
    println!();

    // Part 5: Hodge navigation
    println!("Part 5: Hodge navigation decomposition");
    let nav_flow = vec![1.0, 0.5, 0.0, -0.5]; // example flow
    let _nav = hodge_decompose(&dungeon, &nav_flow);
    println!("  Navigation decomposed into gradient + curl + harmonic");
    println!();

    // Part 6: Lostness
    println!("Part 6: Lostness metric");
    let visits = vec![3.0, 5.0, 1.0, 2.0]; // visit counts per room
    let report = LostnessReport::analyze(&dungeon, &visits, entrance, treasury);
    println!("  Lostness report computed");
}
