use serde::{Deserialize, Serialize};

/// Core balanced ternary value for room connections.
///
/// - `Wall` (-1): impassable barrier, an exit that leads nowhere
/// - `Threshold` (0): a doorway, the neutral element of composition
/// - `Portal` (+1): a wormhole/teleport, creates non-trivial topology
///
/// Composition uses Z₃ addition (mod 3), making this a proper abelian group:
/// - Threshold (0) is the identity
/// - Wall and Portal are inverses
/// - Associative by construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ternary {
    Wall = -1,
    Threshold = 0,
    Portal = 1,
}

impl Ternary {
    /// Numeric value: -1, 0, or +1.
    pub fn value(self) -> i8 {
        self as i8
    }

    /// From numeric value. Clamps to nearest ternary.
    pub fn from_value(v: i8) -> Self {
        match v {
            ..=-1 => Ternary::Wall,
            0 => Ternary::Threshold,
            1.. => Ternary::Portal,
        }
    }

    /// Z₃ composition (mod 3 addition). This is a proper abelian group:
    /// - Identity: Threshold (0)
    /// - Associative by construction
    /// - Wall and Portal are inverses
    /// - Wall + Wall = Portal, Portal + Portal = Wall
    pub fn compose(self, other: Self) -> Self {
        let a = self.value() as i32;
        let b = other.value() as i32;
        let sum = ((a + b) % 3 + 3) % 3; // proper mod 3
        // Map: 0 → Threshold, 1 → Portal, 2 → Wall (since 2 ≡ -1 mod 3)
        match sum {
            0 => Ternary::Threshold,
            1 => Ternary::Portal,
            2 => Ternary::Wall,
            _ => unreachable!(),
        }
    }

    /// Kleene star: transitive closure of a ternary value.
    pub fn kleene_star(self) -> Self {
        self
    }

    /// Additive composition (for parallel paths): max-like.
    pub fn parallel(self, other: Self) -> Self {
        Self::from_value(self.value().max(other.value()))
    }

    /// Inverse in Z₃: Wall ↔ Portal, Threshold is self-inverse.
    pub fn inverse(self) -> Self {
        match self {
            Ternary::Wall => Ternary::Portal,
            Ternary::Threshold => Ternary::Threshold,
            Ternary::Portal => Ternary::Wall,
        }
    }

    /// Negate: flip sign.
    pub fn negate(self) -> Self {
        match self {
            Ternary::Wall => Ternary::Portal,
            Ternary::Threshold => Ternary::Threshold,
            Ternary::Portal => Ternary::Wall,
        }
    }
}

impl std::ops::Add for Ternary {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.compose(rhs)
    }
}

impl std::ops::Mul for Ternary {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // Sign multiplication
        match (self, rhs) {
            (Ternary::Threshold, _) | (_, Ternary::Threshold) => Ternary::Threshold,
            (Ternary::Wall, Ternary::Wall) => Ternary::Portal,
            (Ternary::Wall, Ternary::Portal) | (Ternary::Portal, Ternary::Wall) => Ternary::Wall,
            (Ternary::Portal, Ternary::Portal) => Ternary::Portal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ternary_values() {
        assert_eq!(Ternary::Wall.value(), -1);
        assert_eq!(Ternary::Threshold.value(), 0);
        assert_eq!(Ternary::Portal.value(), 1);
    }

    #[test]
    fn ternary_identity() {
        assert_eq!(Ternary::Threshold.compose(Ternary::Wall), Ternary::Wall);
        assert_eq!(Ternary::Threshold.compose(Ternary::Portal), Ternary::Portal);
        assert_eq!(Ternary::Wall.compose(Ternary::Threshold), Ternary::Wall);
        assert_eq!(Ternary::Portal.compose(Ternary::Threshold), Ternary::Portal);
        assert_eq!(Ternary::Threshold.compose(Ternary::Threshold), Ternary::Threshold);
    }

    #[test]
    fn ternary_inverses() {
        assert_eq!(Ternary::Wall.compose(Ternary::Portal), Ternary::Threshold);
        assert_eq!(Ternary::Portal.compose(Ternary::Wall), Ternary::Threshold);
        assert_eq!(Ternary::Threshold.compose(Ternary::Threshold), Ternary::Threshold);
    }

    #[test]
    fn ternary_associativity() {
        // Z₃ addition is associative for all combinations
        let vals = [Ternary::Wall, Ternary::Threshold, Ternary::Portal];
        for a in &vals {
            for b in &vals {
                for c in &vals {
                    assert_eq!(
                        a.compose(b.compose(*c)),
                        a.compose(*b).compose(*c),
                        "Associativity failed for ({:?}, {:?}, {:?})", a, b, c
                    );
                }
            }
        }
    }

    #[test]
    fn ternary_z3_table() {
        // Z₃ addition table: Wall+Wall=Portal, Portal+Portal=Wall
        assert_eq!(Ternary::Wall.compose(Ternary::Wall), Ternary::Portal);
        assert_eq!(Ternary::Portal.compose(Ternary::Portal), Ternary::Wall);
        assert_eq!(Ternary::Wall.compose(Ternary::Portal), Ternary::Threshold);
        assert_eq!(Ternary::Portal.compose(Ternary::Wall), Ternary::Threshold);
    }

    #[test]
    fn ternary_multiplication() {
        assert_eq!(Ternary::Wall * Ternary::Wall, Ternary::Portal);
        assert_eq!(Ternary::Wall * Ternary::Portal, Ternary::Wall);
        assert_eq!(Ternary::Portal * Ternary::Portal, Ternary::Portal);
        assert_eq!(Ternary::Threshold * Ternary::Wall, Ternary::Threshold);
    }

    #[test]
    fn ternary_negate() {
        assert_eq!(Ternary::Wall.negate(), Ternary::Portal);
        assert_eq!(Ternary::Portal.negate(), Ternary::Wall);
        assert_eq!(Ternary::Threshold.negate(), Ternary::Threshold);
    }

    #[test]
    fn ternary_inverse_property() {
        // x.compose(inverse(x)) = Threshold for all x
        assert_eq!(Ternary::Wall.compose(Ternary::Wall.inverse()), Ternary::Threshold);
        assert_eq!(Ternary::Portal.compose(Ternary::Portal.inverse()), Ternary::Threshold);
        assert_eq!(Ternary::Threshold.compose(Ternary::Threshold.inverse()), Ternary::Threshold);
    }

    #[test]
    fn ternary_commutativity() {
        assert_eq!(Ternary::Wall.compose(Ternary::Portal), Ternary::Portal.compose(Ternary::Wall));
        assert_eq!(Ternary::Wall.compose(Ternary::Wall), Ternary::Portal);
    }
}
