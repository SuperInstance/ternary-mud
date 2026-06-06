use serde::{Deserialize, Serialize};
use crate::dungeon::Dungeon;
use crate::hodge_navigation::traversal_decompose;

/// Lostness component classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LostnessComponent {
    /// Player is on the gradient path toward their goal.
    OnPath,
    /// Player is circulating — harmonic component dominates.
    Circling,
    /// Player is in a dead-end — curl component dominates.
    DeadEnd,
}

/// A report on player lostness state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LostnessReport {
    /// Lostness score: 0.0 = perfectly on path, 1.0 = maximally lost.
    pub score: f64,
    /// Which component dominates the player's traversal.
    pub component: LostnessComponent,
    /// Suggested exit (room index) that would move toward the goal.
    pub suggested_exit: Option<usize>,
}

impl LostnessReport {
    /// Analyze a player's traversal pattern to detect lostness.
    ///
    /// Uses Hodge decomposition with dead-end detection:
    /// - Gradient (variation from mean) = productive movement
    /// - Harmonic (constant) = baseline, not counted as lostness
    /// - Curl (dead-end visits) = wasted movement
    ///
    /// Lostness = curl / (gradient + curl) — what fraction of non-trivial
    /// movement is stuck in dead ends.
    pub fn analyze(
        dungeon: &Dungeon,
        visits: &[f64],
        current_room: usize,
        goal_room: usize,
    ) -> Self {
        let n = dungeon.rooms.len();
        if n == 0 || visits.is_empty() {
            return LostnessReport {
                score: 0.0,
                component: LostnessComponent::OnPath,
                suggested_exit: None,
            };
        }

        let decomp = traversal_decompose(dungeon, visits);

        let gradient_energy = decomp.gradient_norm();
        let curl_energy = decomp.curl_norm();
        let harmonic_energy = decomp.harmonic_norm();

        // For connected graphs, harmonic is constant (= mean visits).
        // We compare gradient vs curl for the "active" movement.
        let total_active = gradient_energy + curl_energy;

        let (score, component) = if total_active < 1e-10 {
            // No gradient or curl: uniform visits
            // On a ring (genus > 0) this means player is circling
            if harmonic_energy > 0.01 && dungeon.genus() > 0 {
                (1.0, LostnessComponent::Circling)
            } else {
                (0.0, LostnessComponent::OnPath)
            }
        } else {
            let curl_frac = curl_energy / total_active;
            let score = curl_frac;

            let component = if curl_frac > 0.5 {
                LostnessComponent::DeadEnd
            } else {
                // Has directional gradient movement → on path
                LostnessComponent::OnPath
            };

            (score, component)
        };

        let suggested_exit = if current_room != goal_room {
            dungeon.shortest_path(current_room, goal_room)
                .and_then(|path| if path.len() > 1 { Some(path[1]) } else { None })
        } else {
            None
        };

        LostnessReport {
            score: score.clamp(0.0, 1.0),
            component,
            suggested_exit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_path_player_low_lostness() {
        // Gradient flow: visits increase monotonically, no dead-end visits
        let d = Dungeon::linear(5);
        // Interior rooms 1,2,3 have degree 2; dead-end rooms 0,4 have degree 1
        // If we avoid the dead ends:
        let visits = vec![0.0, 1.0, 2.0, 3.0, 0.0];
        let report = LostnessReport::analyze(&d, &visits, 2, 4);
        assert!(report.score < 0.1, "On-path score = {}", report.score);
        assert_eq!(report.component, LostnessComponent::OnPath);
    }

    #[test]
    fn circling_player_detected() {
        // Uniform visits on ring = purely harmonic = circling
        let d = Dungeon::ring(4);
        let visits = vec![1.0, 1.0, 1.0, 1.0];
        let report = LostnessReport::analyze(&d, &visits, 0, 2);
        assert_eq!(report.component, LostnessComponent::Circling);
    }

    #[test]
    fn dead_end_player_detected() {
        // Player stuck in dead-end rooms of linear dungeon
        let d = Dungeon::linear(5);
        let visits = vec![10.0, 0.0, 0.0, 0.0, 10.0];
        let report = LostnessReport::analyze(&d, &visits, 4, 0);
        assert_eq!(report.component, LostnessComponent::DeadEnd);
    }

    #[test]
    fn suggested_exit_points_toward_goal() {
        let d = Dungeon::linear(5);
        let visits = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let report = LostnessReport::analyze(&d, &visits, 2, 4);
        assert_eq!(report.suggested_exit, Some(3));
    }

    #[test]
    fn suggested_exit_from_dead_end() {
        let d = Dungeon::linear(5);
        let visits = vec![1.0, 0.0, 0.0, 0.0, 5.0];
        let report = LostnessReport::analyze(&d, &visits, 4, 0);
        assert_eq!(report.suggested_exit, Some(3));
    }

    #[test]
    fn at_goal_no_suggestion() {
        let d = Dungeon::linear(3);
        let visits = vec![1.0, 2.0, 3.0];
        let report = LostnessReport::analyze(&d, &visits, 2, 2);
        assert_eq!(report.suggested_exit, None);
    }

    #[test]
    fn lostness_score_bounded() {
        let d = Dungeon::ring(4);
        let visits = vec![10.0, 10.0, 10.0, 10.0];
        let report = LostnessReport::analyze(&d, &visits, 0, 2);
        assert!(report.score >= 0.0 && report.score <= 1.0);
    }

    #[test]
    fn path_players_have_low_score() {
        // Monotonic visits through a linear dungeon, no dead-end time
        let d = Dungeon::linear(6);
        let visits = vec![0.0, 1.0, 2.0, 3.0, 4.0, 0.0];
        let report = LostnessReport::analyze(&d, &visits, 3, 5);
        assert!(report.score < 0.1, "Score = {}", report.score);
    }
}
