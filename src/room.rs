use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::ternary_lattice::Ternary;

/// A room in the dungeon — the universal context container.
///
/// Each room has:
/// - A unique identifier
/// - Named exits mapping to (target_room_index, connection_type)
/// - An atmosphere value (harmonic function)
/// - Contents (items, descriptions, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub exits: HashMap<String, (usize, Ternary)>,
    pub atmosphere: f64,
    pub contents: Vec<String>,
}

impl Room {
    pub fn new(id: impl Into<String>) -> Self {
        Room {
            id: id.into(),
            exits: HashMap::new(),
            atmosphere: 0.0,
            contents: Vec::new(),
        }
    }

    pub fn with_exit(mut self, name: impl Into<String>, target: usize, conn: Ternary) -> Self {
        self.exits.insert(name.into(), (target, conn));
        self
    }

    pub fn with_atmosphere(mut self, value: f64) -> Self {
        self.atmosphere = value;
        self
    }

    pub fn with_content(mut self, item: impl Into<String>) -> Self {
        self.contents.push(item.into());
        self
    }

    /// Get the ternary connection type to a target room, if an exit exists.
    pub fn connection_to(&self, target: usize) -> Option<Ternary> {
        self.exits.values().find(|(t, _)| *t == target).map(|(_, c)| *c)
    }

    /// List all exit names.
    pub fn exit_names(&self) -> Vec<&str> {
        self.exits.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_new() {
        let r = Room::new("entrance");
        assert_eq!(r.id, "entrance");
        assert!(r.exits.is_empty());
        assert_eq!(r.atmosphere, 0.0);
    }

    #[test]
    fn room_with_exits() {
        let r = Room::new("hall")
            .with_exit("north", 1, Ternary::Threshold)
            .with_exit("portal", 5, Ternary::Portal);
        assert_eq!(r.exits.len(), 2);
        assert_eq!(r.connection_to(1), Some(Ternary::Threshold));
        assert_eq!(r.connection_to(5), Some(Ternary::Portal));
        assert_eq!(r.connection_to(99), None);
    }
}
