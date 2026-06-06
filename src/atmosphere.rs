use crate::dungeon::Dungeon;

/// Solve the Dirichlet problem on the dungeon graph.
///
/// Given boundary rooms with fixed atmosphere values, compute the interior
/// atmosphere by solving the discrete Laplacian equation:
///   Δf(v) = 0 for interior vertices
///   f(v) = boundary_value for boundary vertices
///
/// The atmosphere at each interior room is the average of its neighbors.
/// This uses Gaussian elimination (no external deps).

/// Mark which rooms are boundary (fixed) vs interior (to solve).
pub fn solve_dirichlet(
    dungeon: &Dungeon,
    boundary: &[(usize, f64)],
) -> Vec<f64> {
    let n = dungeon.rooms.len();
    if n == 0 { return vec![]; }

    let mut is_boundary = vec![false; n];
    let mut values = vec![0.0; n];

    for &(idx, val) in boundary {
        is_boundary[idx] = true;
        values[idx] = val;
    }

    // Count interior variables
    let interior: Vec<usize> = (0..n).filter(|&i| !is_boundary[i]).collect();
    let m = interior.len();

    if m == 0 {
        return values;
    }

    // Map room index to variable index
    let mut var_index = vec![usize::MAX; n];
    for (vi, &room_idx) in interior.iter().enumerate() {
        var_index[room_idx] = vi;
    }

    // Build system: for each interior vertex i,
    // degree(i) * f(i) - sum_neighbors f(j) = -sum_boundary_neighbors boundary_value(j)
    let mut a = vec![vec![0.0; m]; m];
    let mut b = vec![0.0; m];

    for (vi, &room_idx) in interior.iter().enumerate() {
        let mut degree = 0.0;
        for j in 0..n {
            if room_idx == j { continue; }
            if dungeon.adjacency[room_idx][j] != crate::ternary_lattice::Ternary::Wall {
                degree += 1.0;
                if is_boundary[j] {
                    b[vi] += values[j];
                } else {
                    a[vi][var_index[j]] = -1.0;
                }
            }
        }
        a[vi][vi] = degree;
    }

    // Solve Ax = b via Gaussian elimination with partial pivoting
    let x = gaussian_elimination(&a, &b);

    for (vi, &room_idx) in interior.iter().enumerate() {
        values[room_idx] = x[vi];
    }

    values
}

/// Gaussian elimination with partial pivoting, from scratch.
/// Solves Ax = b where A is m x m.
pub(crate) fn gaussian_elimination(a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    let m = b.len();
    if m == 0 { return vec![]; }

    // Augmented matrix
    let mut aug = vec![vec![0.0; m + 1]; m];
    for i in 0..m {
        for j in 0..m {
            aug[i][j] = a[i][j];
        }
        aug[i][m] = b[i];
    }

    // Forward elimination with partial pivoting
    for col in 0..m {
        // Find pivot
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..m {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }

        // Swap rows
        if max_row != col {
            aug.swap(col, max_row);
        }

        let pivot = aug[col][col];
        if pivot.abs() < 1e-12 { continue; }

        // Eliminate below
        for row in (col + 1)..m {
            let factor = aug[row][col] / pivot;
            for j in col..=m {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; m];
    for i in (0..m).rev() {
        let mut sum = aug[i][m];
        for j in (i + 1)..m {
            sum -= aug[i][j] * x[j];
        }
        if aug[i][i].abs() > 1e-12 {
            x[i] = sum / aug[i][i];
        }
    }

    x
}

/// Verify that the discrete Laplacian is zero at all interior points.
pub fn verify_laplacian(dungeon: &Dungeon, values: &[f64], boundary: &[(usize, bool)]) -> Vec<f64> {
    let n = dungeon.rooms.len();
    let mut residuals = vec![0.0; n];

    for i in 0..n {
        if boundary.iter().any(|&(idx, _)| idx == i) { continue; }
        let mut degree = 0.0;
        let mut neighbor_sum = 0.0;
        for j in 0..n {
            if i == j { continue; }
            if dungeon.adjacency[i][j] != crate::ternary_lattice::Ternary::Wall {
                degree += 1.0;
                neighbor_sum += values[j];
            }
        }
        if degree > 0.0 {
            residuals[i] = degree * values[i] - neighbor_sum;
        }
    }

    residuals
}

/// Compute the discrete Laplacian at each room.
pub fn discrete_laplacian(dungeon: &Dungeon, values: &[f64]) -> Vec<f64> {
    let n = dungeon.rooms.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let mut degree = 0.0;
        let mut neighbor_sum = 0.0;
        for j in 0..n {
            if i == j { continue; }
            if dungeon.adjacency[i][j] != crate::ternary_lattice::Ternary::Wall {
                degree += 1.0;
                neighbor_sum += values[j];
            }
        }
        result[i] = degree * values[i] - neighbor_sum;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Room;
    use crate::ternary_lattice::Ternary;

    #[test]
    fn atmosphere_linear_interpolation() {
        // Linear dungeon: 0 - 1 - 2 - 3 - 4
        // Boundary: room 0 = 0.0, room 4 = 4.0
        // Interior should be 1.0, 2.0, 3.0
        let d = Dungeon::linear(5);
        let values = solve_dirichlet(&d, &[(0, 0.0), (4, 4.0)]);
        assert!((values[1] - 1.0).abs() < 1e-6);
        assert!((values[2] - 2.0).abs() < 1e-6);
        assert!((values[3] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn atmosphere_satisfies_laplacian() {
        let d = Dungeon::linear(5);
        let values = solve_dirichlet(&d, &[(0, 0.0), (4, 4.0)]);
        let lap = discrete_laplacian(&d, &values);
        // Laplacian should be ~0 at interior rooms (1, 2, 3)
        for i in 1..4 {
            assert!(lap[i].abs() < 1e-6, "Room {} laplacian = {}", i, lap[i]);
        }
    }

    #[test]
    fn atmosphere_single_boundary() {
        let mut d = Dungeon::new();
        d.add_room(Room::new("a"));
        d.add_room(Room::new("b"));
        d.connect_bidir(0, 1, Ternary::Threshold);
        let values = solve_dirichlet(&d, &[(0, 5.0)]);
        assert!((values[0] - 5.0).abs() < 1e-6);
        // Room 1 has no boundary, is average of neighbors = just room 0
        assert!((values[1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn atmosphere_ring_symmetric() {
        // Ring of 4 rooms, boundary at 0 and 2
        let d = Dungeon::ring(4);
        let values = solve_dirichlet(&d, &[(0, 1.0), (2, -1.0)]);
        // By symmetry: room 1 = 0.0, room 3 = 0.0
        assert!((values[1] - 0.0).abs() < 1e-6);
        assert!((values[3] - 0.0).abs() < 1e-6);
    }
}
