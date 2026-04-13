use crate::{Ruleset, State};

use rand::prelude::*;
use std::cmp::Ordering;

/// One of the two players in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    fn other(self) -> Self {
        match self {
            Self::P1 => Self::P2,
            Self::P2 => Self::P1,
        }
    }
}

/// The result of a single player's turn.
enum TurnResult {
    Bust,
    Stand,
    Roll(u32),
}

fn play_turn(rng: &mut SmallRng, ruleset: Ruleset, state: State, n: u32) -> TurnResult {
    if n == 0 {
        return TurnResult::Stand;
    }
    let total: u32 = (0..n).map(|_| rng.random_range(1..=ruleset.sides())).sum();
    if state.active() + total > ruleset.max() {
        TurnResult::Bust
    } else {
        TurnResult::Roll(state.active() + total)
    }
}

/// Start playing a game from a given state.
pub fn play_game_from_state(
    start: State,
    p1_strategy: impl Fn(State) -> u32,
    p2_strategy: impl Fn(State) -> u32,
    ruleset: Ruleset,
    rng: &mut SmallRng,
) -> (Option<Player>, Vec<(State, Player)>) {
    // We reconstruct a consistent game by assigning P1 as the active player
    // and P2 as the queued player at the starting state.
    let mut p1_score = start.active();
    let mut p2_score = start.queued();
    let mut last_round = start.last();
    let mut active = Player::P1;
    let mut visited: Vec<(State, Player)> = vec![];

    let winner = loop {
        let (active_score, queued_score) = match active {
            Player::P1 => (p1_score, p2_score),
            Player::P2 => (p2_score, p1_score),
        };

        let state = State::new(active_score, queued_score, last_round);
        visited.push((state, active));

        let n = match active {
            Player::P1 => p1_strategy(state),
            Player::P2 => p2_strategy(state),
        };

        match play_turn(rng, ruleset, state, n) {
            TurnResult::Bust => {
                break Some(active.other());
            }
            TurnResult::Stand if last_round => {
                let winner = match p1_score.cmp(&p2_score) {
                    Ordering::Greater => Some(Player::P1),
                    Ordering::Less => Some(Player::P2),
                    Ordering::Equal => None,
                };
                break winner;
            }
            TurnResult::Stand => {
                last_round = true;
                active = active.other();
            }
            TurnResult::Roll(active_score) if last_round => {
                match active {
                    Player::P1 => p1_score = active_score,
                    Player::P2 => p2_score = active_score,
                }
                let winner = match p1_score.cmp(&p2_score) {
                    Ordering::Greater => Some(Player::P1),
                    Ordering::Less => Some(Player::P2),
                    Ordering::Equal => None,
                };
                break winner;
            }
            TurnResult::Roll(active_score) => {
                match active {
                    Player::P1 => p1_score = active_score,
                    Player::P2 => p2_score = active_score,
                }
                active = active.other();
            }
        }
    };

    (winner, visited)
}
