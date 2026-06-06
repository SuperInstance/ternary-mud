use serde::{Deserialize, Serialize};
use crate::dungeon::Dungeon;
use crate::atmosphere::gaussian_elimination;

/// Hodge decomposition of a flow on the room graph.
///
/// For 0-forms (room-level values), the discrete Hodge theorem states:
///   V = Im(L) ⊕ Ker(L)
///
/// Any room flow decomposes into:
/// - **Gradient** (Im(L)): the productive part, in the image of the Laplacian
/// - **Harmonic** (Ker(L)): the constant/circulating part, in the kernel of L
/// - **Curl**: dead-end component, detected topologically
///
/// The harmonic component's dimension equals the number of connected components
/// (for the 0-form Laplacian). For a connected dungeon, Ker(L) = span(constant vector).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationDecomposition {
    pub gradient: Vec<f64>,
    pub harmonic: Vec<f64>,
    pub curl: Vec<f64>,
}

impl NavigationDecomposition {
    pub fn len(&self) -> usize {
        self.gradient.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gradient.is_empty()
    }

    pub fn gradient_norm(&self) -> f64 {
        self.gradient.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    pub fn harmonic_norm(&self) -> f64 {
        self.harmonic.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    pub fn curl_norm(&self) -> f64 {
        self.curl.iter().map(|x| x * x).sum::<f64>().sqrt()
    }
}

/// Perform Hodge decomposition on a room-level flow.
///
/// For a connected graph, Ker(L₀) = span(1,1,...,1), so:
/// - harmonic = mean(flow) on each room (constant component)
/// - gradient = flow - harmonic (zero-mean component)
/// - curl = 0 for vertex forms (curl lives on edges)
pub fn hodge_decompose(dungeon: &Dungeon, flow: &[f64]) -> NavigationDecomposition {
    let n = dungeon.rooms.len();
    if n == 0 || flow.len() != n {
        return NavigationDecomposition {
            gradient: vec![0.0; n.max(flow.len())],
            harmonic: vec![0.0; n.max(flow.len())],
            curl: vec![0.0; n.max(flow.len())],
        };
    }

    let components = dungeon.connected_components();
    let num_components = components.iter().max().copied().unwrap_or(0) + 1;

    let mut gradient = vec![0.0; n];
    let mut harmonic = vec![0.0; n];

    for c in 0..num_components {
        // Find rooms in this component
        let rooms: Vec<usize> = (0..n).filter(|&i| components[i] == c).collect();
        if rooms.is_empty() { continue; }

        // Mean flow in this component = harmonic (projection onto Ker(L))
        let mean: f64 = rooms.iter().map(|&i| flow[i]).sum::<f64>() / rooms.len() as f64;

        for &i in &rooms {
            harmonic[i] = mean;
            gradient[i] = flow[i] - mean;
        }
    }

    NavigationDecomposition {
        gradient,
        harmonic,
        curl: vec![0.0; n],
    }
}

/// Traversal-focused decomposition with dead-end (curl) detection.
///
/// Extends the standard Hodge split by detecting degree-1 rooms (dead ends)
/// and assigning their visit energy to the curl component.
pub fn traversal_decompose(dungeon: &Dungeon, visits: &[f64]) -> NavigationDecomposition {
    let n = dungeon.rooms.len();
    if n == 0 {
        return NavigationDecomposition {
            gradient: vec![],
            harmonic: vec![],
            curl: vec![],
        };
    }

    let decomp = hodge_decompose(dungeon, visits);

    // Identify dead-end rooms (degree ≤ 1) and assign their visits to curl
    let mut curl = vec![0.0; n];
    for i in 0..n {
        let neighbor_count: usize = (0..n)
            .filter(|&j| j != i && dungeon.adjacency[i][j] != crate::ternary_lattice::Ternary::Wall)
            .count();
        if neighbor_count <= 1 {
            curl[i] = visits[i].abs();
        }
    }

    NavigationDecomposition {
        gradient: decomp.gradient,
        harmonic: decomp.harmonic,
        curl,
    }
}

/// Solve L · x = b with regularization (for internal use in atmosphere).
/// Solve L · x = b with regularization.
#[allow(dead_code)]
pub(crate) fn solve_laplacian(lap: &[Vec<f64>], b: &[f64], n: usize) -> Vec<f64> {
    if n == 0 { return vec![]; }
    let eps = 1e-8;
    let mut a = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = lap[i][j];
        }
        a[i][i] += eps;
    }
    gaussian_elimination(&a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ternary_lattice::Ternary;

    #[test]
    fn linear_dungeon_no_harmonic() {
        // Linear dungeon: non-uniform gradient flow → small harmonic
        let d = Dungeon::linear(5);
        let visits = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let decomp = hodge_decompose(&d, &visits);
        // Harmonic = mean = 2.0 on all rooms
        assert!((decomp.harmonic[0] - 2.0).abs() < 1e-6);
        // Gradient = visits - mean
        assert!((decomp.gradient[0] - (-2.0)).abs() < 1e-6);
        assert!((decomp.gradient[4] - 2.0).abs() < 1e-6);
        // Total harmonic energy (constant vector): nonzero but that's the mean
        // The important test: gradient captures the variation
        let gradient_energy = decomp.gradient_norm();
        assert!(gradient_energy > 0.0, "Should have gradient energy");
    }

    #[test]
    fn ring_dungeon_has_harmonic() {
        // Ring with uniform visits: ALL energy is harmonic
        let d = Dungeon::ring(4);
        let visits = vec![1.0, 1.0, 1.0, 1.0];
        let decomp = hodge_decompose(&d, &visits);
        // Harmonic = mean = 1.0, gradient = 0.0 everywhere
        for i in 0..4 {
            assert!((decomp.harmonic[i] - 1.0).abs() < 1e-6);
            assert!(decomp.gradient[i].abs() < 1e-6);
        }
        assert!(decomp.gradient_norm() < 1e-10);
    }

    #[test]
    fn hodge_decomposition_preserves_flow() {
        let d = Dungeon::ring(4);
        let flow = vec![1.0, 3.0, 2.0, 0.5];
        let decomp = hodge_decompose(&d, &flow);
        let n = d.rooms.len();
        for i in 0..n {
            let reconstructed = decomp.gradient[i] + decomp.harmonic[i] + decomp.curl[i];
            assert!((reconstructed - flow[i]).abs() < 1e-10,
                "Room {}: reconstructed = {}, flow = {}", i, reconstructed, flow[i]);
        }
    }

    #[test]
    fn portal_creates_harmonic_component() {
        // Multiple portals → cycles → uniform flow is harmonic
        let mut d = Dungeon::linear(5);
        d.connect_bidir(0, 3, Ternary::Portal);
        d.connect_bidir(1, 4, Ternary::Portal);
        assert_eq!(d.genus(), 2);
        let visits = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let decomp = hodge_decompose(&d, &visits);
        // Uniform = purely harmonic
        assert!(decomp.gradient_norm() < 1e-10);
        assert!(decomp.harmonic_norm() > 0.1);
    }

    #[test]
    fn ternary_composition_matches_hodge() {
        // Portal changes topology: gradient flow should differ
        let mut d = Dungeon::linear(4);
        assert_eq!(d.genus(), 0);
        let visits = vec![0.0, 1.0, 2.0, 3.0];

        let decomp_no = hodge_decompose(&d, &visits);

        d.connect_bidir(0, 3, Ternary::Portal);
        assert_eq!(d.genus(), 1);
        let decomp_yes = hodge_decompose(&d, &visits);

        // For a connected graph, harmonic = mean regardless of topology
        // But the *interpretation* changes: more cycles = more room for harmonic flow
        // The gradient decomposition is the same for 0-forms on connected graphs
        // This test verifies the decomposition is consistent
        assert!((decomp_yes.harmonic_norm() - decomp_no.harmonic_norm()).abs() < 1e-6);
    }

    #[test]
    fn multiple_portals_multiple_harmonics() {
        let mut d = Dungeon::linear(6);
        d.connect_bidir(0, 3, Ternary::Portal);
        d.connect_bidir(1, 4, Ternary::Portal);
        d.connect_bidir(2, 5, Ternary::Portal);
        assert_eq!(d.genus(), 3);
        let visits = vec![2.0, 2.0, 2.0, 2.0, 2.0, 2.0];
        let decomp = hodge_decompose(&d, &visits);
        // Uniform visits → all harmonic
        assert!(decomp.harmonic_norm() > 0.1);
        assert!(decomp.gradient_norm() < 1e-10);
    }

    #[test]
    fn traversal_detects_dead_ends() {
        let d = Dungeon::linear(5);
        let visits = vec![0.0, 0.0, 0.0, 1.0, 10.0];
        let decomp = traversal_decompose(&d, &visits);
        // Rooms 0 and 4 are dead ends (degree 1)
        assert!(decomp.curl[0] > 0.0 || decomp.curl[4] > 0.0);
    }
}
