use serde::{Deserialize, Serialize};
use crate::ternary_lattice::Ternary;
use crate::room::Room;

/// A dungeon as a category: objects are rooms, morphisms are doors.
///
/// The adjacency matrix stores ternary connection types.
/// Topology is derived from the structure: connectivity, genus, fundamental group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub rooms: Vec<Room>,
    pub adjacency: Vec<Vec<Ternary>>,
}

impl Dungeon {
    pub fn new() -> Self {
        Dungeon {
            rooms: Vec::new(),
            adjacency: Vec::new(),
        }
    }

    /// Add a room, returning its index.
    pub fn add_room(&mut self, room: Room) -> usize {
        let idx = self.rooms.len();
        self.rooms.push(room);
        // Extend adjacency matrix
        for row in &mut self.adjacency {
            row.push(Ternary::Wall);
        }
        let mut new_row = vec![Ternary::Wall; idx + 1];
        new_row[idx] = Ternary::Threshold; // self-connection is threshold (identity)
        self.adjacency.push(new_row);
        idx
    }

    /// Set a directed connection from `from` to `to`.
    pub fn connect(&mut self, from: usize, to: usize, conn: Ternary) {
        self.adjacency[from][to] = conn;
    }

    /// Set a bidirectional connection.
    pub fn connect_bidir(&mut self, a: usize, b: usize, conn: Ternary) {
        self.adjacency[a][b] = conn;
        self.adjacency[b][a] = conn;
    }

    /// Number of rooms.
    pub fn len(&self) -> usize {
        self.rooms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }

    /// Compute connectivity: find connected components (ignoring Wall).
    /// Returns a vector of component IDs, one per room.
    pub fn connected_components(&self) -> Vec<usize> {
        let n = self.rooms.len();
        if n == 0 { return vec![]; }
        let mut component = vec![usize::MAX; n];
        let mut current = 0;
        for start in 0..n {
            if component[start] != usize::MAX { continue; }
            // BFS
            let mut queue = vec![start];
            component[start] = current;
            let mut head = 0;
            while head < queue.len() {
                let node = queue[head];
                head += 1;
                for (j, &conn) in self.adjacency[node].iter().enumerate() {
                    if component[j] == usize::MAX && conn != Ternary::Wall {
                        component[j] = current;
                        queue.push(j);
                    }
                }
            }
            current += 1;
        }
        component
    }

    /// Is the dungeon fully connected?
    pub fn is_connected(&self) -> bool {
        let components = self.connected_components();
        components.iter().all(|&c| c == 0)
    }

    /// Count portal connections (non-trivial topology generators).
    pub fn portal_count(&self) -> usize {
        let n = self.rooms.len();
        let mut count = 0;
        for i in 0..n {
            for j in 0..n {
                if self.adjacency[i][j] == Ternary::Portal && i < j {
                    count += 1;
                }
            }
        }
        count
    }

    /// Compute genus: cyclomatic number = |E| - |V| + |C|.
    /// This counts independent cycles in the dungeon graph.
    pub fn genus(&self) -> usize {
        let n = self.rooms.len();
        if n == 0 { return 0; }
        let components = self.connected_components();
        let num_components = *components.iter().max().unwrap_or(&0) + 1;
        let mut edges: isize = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                if self.adjacency[i][j] != Ternary::Wall {
                    edges += 1;
                }
            }
        }
        let g = edges - n as isize + num_components as isize;
        g.max(0) as usize
    }

    /// Build the graph Laplacian: L = D - A (as f64 matrix).
    /// A[i][j] = 1.0 if connected (not Wall), D = degree diagonal.
    pub fn laplacian(&self) -> Vec<Vec<f64>> {
        let n = self.rooms.len();
        let mut l = vec![vec![0.0; n]; n];
        for i in 0..n {
            let mut degree = 0.0;
            for j in 0..n {
                if i != j && self.adjacency[i][j] != Ternary::Wall {
                    let weight = self.adjacency[i][j].value() as f64;
                    l[i][j] = -weight;
                    degree += weight;
                }
            }
            l[i][i] = degree;
        }
        l
    }

    /// Build the degree-weighted incidence matrix (for Hodge decomposition).
    /// Returns B: m x n matrix where m = number of directed edges, n = number of rooms.
    pub fn incidence_matrix(&self) -> (Vec<(usize, usize)>, Vec<Vec<f64>>) {
        let n = self.rooms.len();
        let mut edges = Vec::new();
        for i in 0..n {
            for j in 0..n {
                if i != j && self.adjacency[i][j] != Ternary::Wall {
                    edges.push((i, j));
                }
            }
        }
        let m = edges.len();
        let mut b = vec![vec![0.0; n]; m];
        for (ei, (i, j)) in edges.iter().enumerate() {
            b[ei][*i] = 1.0;
            b[ei][*j] = -1.0;
        }
        (edges, b)
    }

    /// Find shortest path (BFS) from `start` to `end`, returning room indices.
    pub fn shortest_path(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        if start == end { return Some(vec![start]); }
        let n = self.rooms.len();
        let mut visited = vec![false; n];
        let mut parent = vec![usize::MAX; n];
        let mut queue = vec![start];
        visited[start] = true;
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head];
            head += 1;
            for j in 0..n {
                if !visited[j] && self.adjacency[node][j] != Ternary::Wall {
                    visited[j] = true;
                    parent[j] = node;
                    if j == end {
                        // Reconstruct path
                        let mut path = vec![end];
                        let mut cur = end;
                        while cur != start {
                            cur = parent[cur];
                            path.push(cur);
                        }
                        path.reverse();
                        return Some(path);
                    }
                    queue.push(j);
                }
            }
        }
        None
    }
}

impl Default for Dungeon {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Dungeon builders for testing ----

impl Dungeon {
    /// Build a linear dungeon: room 0 -- 1 -- 2 -- ... -- (n-1)
    pub fn linear(n: usize) -> Self {
        let mut d = Dungeon::new();
        for i in 0..n {
            d.add_room(Room::new(format!("room_{}", i)));
        }
        for i in 0..n.saturating_sub(1) {
            d.connect_bidir(i, i + 1, Ternary::Threshold);
        }
        d
    }

    /// Build a ring dungeon: room 0 -- 1 -- 2 -- ... -- (n-1) -- 0
    pub fn ring(n: usize) -> Self {
        let mut d = Self::linear(n);
        if n > 2 {
            d.connect_bidir(0, n - 1, Ternary::Threshold);
        }
        d
    }

    /// Build a tree dungeon (binary tree of given depth).
    pub fn tree(depth: usize) -> Self {
        let node_count = (1 << (depth + 1)) - 1;
        let mut d = Dungeon::new();
        for i in 0..node_count {
            d.add_room(Room::new(format!("tree_{}", i)));
        }
        for i in 0..node_count {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            if left < node_count {
                d.connect_bidir(i, left, Ternary::Threshold);
            }
            if right < node_count {
                d.connect_bidir(i, right, Ternary::Threshold);
            }
        }
        d
    }

    /// Build a dungeon with a dead-end branch.
    pub fn dead_end_dungeon() -> Self {
        let mut d = Dungeon::new();
        // 0 - 1 - 2 (main path)
        //         \
        //          3 (dead end)
        for i in 0..4 {
            d.add_room(Room::new(format!("de_{}", i)));
        }
        d.connect_bidir(0, 1, Ternary::Threshold);
        d.connect_bidir(1, 2, Ternary::Threshold);
        d.connect_bidir(2, 3, Ternary::Threshold);
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_connectivity_linear() {
        let d = Dungeon::linear(5);
        assert!(d.is_connected());
        assert_eq!(d.genus(), 0); // tree: no cycles
    }

    #[test]
    fn dungeon_connectivity_ring() {
        let d = Dungeon::ring(5);
        assert!(d.is_connected());
        assert_eq!(d.genus(), 1); // one cycle
    }

    #[test]
    fn dungeon_disconnected() {
        let mut d = Dungeon::new();
        d.add_room(Room::new("a"));
        d.add_room(Room::new("b"));
        // No connections
        assert!(!d.is_connected());
        assert_eq!(d.connected_components(), vec![0, 1]);
    }

    #[test]
    fn dungeon_shortest_path_linear() {
        let d = Dungeon::linear(5);
        let path = d.shortest_path(0, 4).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn dungeon_shortest_path_ring() {
        let d = Dungeon::ring(5);
        let path = d.shortest_path(0, 3).unwrap();
        // Shorter through 0-4-3 (length 3) vs 0-1-2-3 (length 4)
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn portal_creates_nontrivial_topology() {
        let mut d = Dungeon::linear(5);
        // Linear 5: 4 edges, 5 vertices, 1 component → genus = 0
        assert_eq!(d.genus(), 0);
        // Add a portal shortcut 0↔4
        d.connect_bidir(0, 4, Ternary::Portal);
        // Now: 5 edges, 5 vertices, 1 component → genus = 1
        assert_eq!(d.genus(), 1);
        assert_eq!(d.portal_count(), 1);
    }
}
