use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::{Add, Mul, Sub};

use csv::Writer;
use memoize::memoize;
use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, ToPrimitive, Zero};

/// A state of Greed.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct State {
    /// The score of the player whose turn it is.
    active: u16,
    /// The score of the player whose turn is up next.
    queued: u16,
    /// Whether this is the last turn of the game.
    last: bool,
}

impl State {
    fn new(active: u16, queued: u16, last: bool) -> Self {
        State {
            active,
            queued,
            last,
        }
    }
}

/// An action to perform, with its corresponding rating.
#[derive(Debug, Copy, Clone, Default)]
struct Action {
    /// Dice to roll
    n: u16,
    /// Rating given a roll of `n` dice.
    rating: f64,
}

impl Action {
    fn new(n: u16, rating: f64) -> Self {
        Action { n, rating }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GreedSolver {
    /// Maximum score allowed.
    max: u16,
    /// The # of sides on each dice.
    sides: u16,
    /// Table of state-action pairs
    table: BTreeMap<State, Action>,
}

impl GreedSolver {
    pub fn new(max: u16, sides: u16) -> Self {
        GreedSolver {
            max,
            sides,
            table: BTreeMap::new(),
        }
    }
    pub fn solve(&mut self) {
        // first solve all the terminal states,
        self.solve_terminal_states();
        // then solve all the normal states (in the correct order)
        self.solve_normal_states();
    }
}

impl GreedSolver {
    // Solve terminal states
    fn solve_terminal_states(&mut self) {
        for turn in 0..=self.max {
            for next in 0..=self.max {
                let state = State::new(turn, next, true);
                let action = self.find_optimal_terminal_action(&state);
                self.table.insert(state, action);
            }
        }
    }
    /// Find the optimal terminal action for a given state.
    fn find_optimal_terminal_action(&self, state: &State) -> Action {
        if state.active > state.queued {
            // If already ahead, doing nothing wins 100% of the time.
            return Action { n: 0, rating: 1.0 };
        }

        let max_reasonable_n: u16 =
            ((self.max + state.queued - 2 * state.active) / (self.sides + 1)) + 1;

        let (optimal_roll, optimal_rating) = (0..=max_reasonable_n)
            .map(|dice_rolled| {
                (dice_rolled, {
                    // println!("{}", self.calc_terminal_rating(state, dice_rolled));
                    self.calc_terminal_rating(state, dice_rolled)
                })
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        Action::new(optimal_roll, optimal_rating)
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice.
    fn calc_terminal_rating(&self, state: &State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            return match state.active.cmp(&state.queued) {
                Ordering::Less => 0.0,
                Ordering::Equal => 0.5,
                Ordering::Greater => 1.0,
            };
        }
        (dice_rolled..=self.sides * dice_rolled).fold(0.0, |acc, dice_total| {
            match (state.active + dice_total).cmp(&state.queued) {
                Ordering::Greater if state.active <= self.max => {
                    acc + pmf(dice_total, dice_rolled, self.sides)
                }
                Ordering::Equal => acc + 0.5 * pmf(dice_total, dice_rolled, self.sides),
                _ => acc,
            }
        })
    }
}

impl GreedSolver {
    // Solve normal states
    fn solve_normal_states(&mut self) {
        for order in 0..=2 * self.max {
            for place in 0..=order.min(2 * self.max - order) {
                // calculate the player and opponent score for this order and place
                let (turn, next) = match order < self.max {
                    true => (place + self.max - order, self.max - place),
                    false => (place, 2 * self.max - order - place),
                };
                let state = State::new(turn, next, false);
                let action = self.find_optimal_normal_action(&state);
                self.table.insert(state, action);
            }
        }
    }
    /// Find the optimal normal action for a given state.
    fn find_optimal_normal_action(&mut self, state: &State) -> Action {
        let max_reasonable_n = 2 * (self.max - state.active) / (self.sides + 1) + 2; // +2 for safety of checking high enough
        let (optimal_roll, optimal_rating) = (0..=max_reasonable_n)
            .map(|dice_rolled| (dice_rolled, self.calc_normal_rating(state, dice_rolled)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        Action::new(optimal_roll, optimal_rating)
    }
    /// Calculate the rating when in state `state` and rolling `dice_rolled` # of dice.
    fn calc_normal_rating(&mut self, state: &State, dice_rolled: u16) -> f64 {
        if dice_rolled == 0 {
            let terminal_state = State::new(state.queued, state.active, true);
            return 1.0 - self.table.get(&terminal_state).unwrap().rating;
        }
        (dice_rolled..=self.sides * dice_rolled).fold(0.0, |acc, dice_total| {
            match state.active + dice_total < self.max {
                true => {
                    let probability: f64 = pmf(dice_total, dice_rolled, self.sides);
                    let state = State::new(state.queued, state.active + dice_total, false);
                    let rating: f64 = 1.0 - self.table.get(&state).unwrap().rating;
                    acc + probability * rating
                }
                false => acc,
            }
        })
    }
}

impl GreedSolver {
    pub fn csv(&self, path: &str) -> Result<(), csv::Error> {
        let mut writer = Writer::from_path(path)?;

        // Write headers
        writer.serialize(("active", "queued", "last", "n", "rating"))?;
        for (state, action) in self.table.clone() {
            writer.serialize((
                state.active,
                state.queued,
                state.last,
                action.n,
                action.rating,
            ))?;
        }
        writer.flush()?;
        Ok(())
    }
    pub fn display(&self) {
        // terminal states
        for (state, action) in self.table.iter().filter(|(state, _)| state.last) {
            println!(
                "({}, {}, terminal) => (dice: #{}, rating: {})",
                state.active, state.queued, action.n, action.rating
            );
        }
        println!();
        // normal states
        for (state, action) in self.table.iter().filter(|(state, _)| !state.last) {
            println!(
                "({}, {}, normal) => (dice: #{}, rating: {})",
                state.active, state.queued, action.n, action.rating
            );
        }
    }
}

#[memoize]
fn pmf(total: u16, n: u16, s: u16) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let mut valid_comb: BigInt = Zero::zero();
    for k in 0..=(total - n) / s {
        let current_comb: BigInt = combinations(n, k).mul(combinations(total - s * k - 1, n - 1));
        if k % 2 == 0 {
            valid_comb = valid_comb.add(current_comb);
        } else {
            valid_comb = valid_comb.sub(current_comb);
        }
    }
    let mut total_comb: BigInt = s.into();
    total_comb = total_comb.pow(n as u32);

    let result: Ratio<BigInt> = Ratio::new(valid_comb, total_comb);
    result.to_f64().unwrap()
}

fn combinations(n: u16, k: u16) -> BigInt {
    let cutoff: u16 = if k < n - k { n - k } else { k };
    (cutoff + 1..=n).fold(One::one(), |acc: BigInt, x| acc.mul(x))
        / (1..=n - cutoff).fold(One::one(), |acc: BigInt, x| acc.mul(x))
}
