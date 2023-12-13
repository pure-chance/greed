// ----- ALLOW DEAD CODE ----- //

#![allow(dead_code)]

// ----- ENVIRONMENT SETUP ----- //

pub const MAX_SCORE: u32 = 100;
pub const SIDES: u32 = 6;

// ----- IMPORTS ----- //
// bigint
use num_bigint::BigInt;
use num_rational::Ratio;
// math stuff -- mostly on bigint
use std::ops::{Mul, Div, Add, Sub};
use std::cmp::min;
use num_traits::{Zero, One, ToPrimitive};
// csv stuff
use std::error::Error;
use csv::Writer;
use derive_getters::Getters;
// memoization
use memoize::memoize;

// ----- PMF ----- //

fn mul_between(min: u32, max: u32) -> BigInt {
    let mut result: BigInt = One::one();
    for i in min..=max {
        result = result.mul(i);
    }
    return result;
}

fn combinations(n: u32, k: u32) -> BigInt {
    let cutoff: u32 = if k < n-k {n-k} else {k};
    let mut combinations: BigInt = mul_between(cutoff+1, n);
    combinations = combinations.div(mul_between(1, n-cutoff));
    return combinations;
}

#[memoize]
fn pmf(total: u32, n: u32, s: u32) -> f64 {
    if total == 0 {return 0.0; }
    let mut valid_comb: BigInt = Zero::zero();
    for k in 0..=(total-n)/s {
        let current_comb: BigInt = combinations(n, k).mul(combinations(total-s*k-1, n-1));
        if k % 2 == 0 { valid_comb = valid_comb.add(current_comb); } 
        else { valid_comb = valid_comb.sub(current_comb); }
    }
    let mut total_comb: BigInt = s.into();
    total_comb = total_comb.pow(n);

    let result: Ratio<BigInt> = Ratio::new(valid_comb, total_comb);
    // println!("combinations = {}, total = {}", valid_comb, total_comb);
    return result.to_f64().unwrap();
}

// ----- SETUP GREED AND STATES ----- //

#[derive(Copy, Clone, Default, Getters)] // IMPORTANT
pub struct State {
    best_n: u32,
    best_rating: f64
}

impl State {
    pub fn get_best_n(&mut self) -> u32 { return self.best_n; }
    pub fn get_best_rating(&mut self) -> f64 { return self.best_rating; }
}

pub struct Greed {
    // the stuff we aim to find
    terminal: [[State; (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize],
    normal: [[State; (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize]
}

// ----- IMPLEMENTATION ----- //

impl Greed {
    pub fn get_state(&mut self, player_score: u32, opponent_score: u32, last_turn: bool) -> State {
        if last_turn { return self.terminal[opponent_score as usize][player_score as usize]; }
        else { return self.normal[opponent_score as usize][player_score as usize]; }
    }
    pub fn set_state(&mut self, player_score: u32, opponent_score: u32, last_turn: bool, result: State) {
        if last_turn { self.terminal[opponent_score as usize][player_score as usize] = result; }
        else { self.normal[opponent_score as usize][player_score as usize] = result; }
    }
    pub fn get_terminal_states(&mut self) -> [[State; (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize] { 
        return self.terminal; 
    }
    pub fn get_normal_states(&mut self) -> [[State; (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize] { 
        return self.normal; 
    }
    pub fn setup() -> Greed {
        return Greed {
            terminal: [[State::default(); (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize],
            normal: [[State::default(); (MAX_SCORE+1) as usize]; (MAX_SCORE+1) as usize],
        };
    }

    // ----- FIND THE OPTIMAL N AND P FOR THE LAST ROLL BOARD ----- //

    fn calculate_terminal_rating(&mut self, player_score: u32, opponent_score: u32, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            if player_score < opponent_score { return 0.0; }
            else if player_score == opponent_score { return 0.5; }
            else { return 1.0; }
        }
        let mut rating: f64 = 0.0;
        for sum_dice in dice_rolled..=SIDES*dice_rolled {
            if opponent_score < player_score + sum_dice && player_score + sum_dice <= MAX_SCORE {
                rating += pmf(sum_dice, dice_rolled, SIDES);
            }
            else if opponent_score == player_score + sum_dice {
                rating += 0.5 * pmf(sum_dice, dice_rolled, SIDES);
            }
            else { continue; }
        }
        return rating;
    }
    pub fn calculate_optimal_terminal_action(&mut self, player_score: u32, opponent_score: u32) -> State {
        if player_score > opponent_score {
            // if you are already ahead, you win 100% of the time by doing nothing
            return State { best_n: 0, best_rating: 1.0 };
        }
        if SIDES * (opponent_score - player_score + 1) <= (MAX_SCORE - player_score) {
            // if you can roll n dice and the entire range is between opponent and MAX_SCORE, you win 100% of the time
            return State { best_n: opponent_score - player_score + 1, best_rating: 1.0 };
        }
        let max_reasonable_n: u32 = ( ( MAX_SCORE + opponent_score - 2*player_score) / (SIDES + 1) ) + 1;
        let mut optimal_roll: u32 = 0; let mut optimal_rating: f64 = 0.0;
        for dice_rolled in 0..=max_reasonable_n {
            let new_rating = self.calculate_terminal_rating(player_score, opponent_score, dice_rolled);
            if new_rating > optimal_rating {
                optimal_roll = dice_rolled;
                optimal_rating = new_rating;
            }
        }
        return State {
            best_n: optimal_roll,
            best_rating: optimal_rating
        };
    }
    pub fn apply_optimal_terminal_actions(&mut self) {
        for player_score in 0..=MAX_SCORE {
            for opponent_score in 0..=MAX_SCORE {
                let result: State = self.calculate_optimal_terminal_action(player_score, opponent_score);
                self.set_state(player_score, opponent_score, true, result);
                // println!(
                //     "player = {}, opponent = {}, best_n = {}, rating = {}", 
                //     player_score, opponent_score, result.best_n, result.best_rating
                // )
            }
        }
    }

    // ----- FIND THE OPTIMAL N AND P FOR THE NORMAL BOARD ----- //

    /*
    For a given normal board state (player_score, opponent_score, 1) and a number of dice rolled (dice_rolled), calculate the rating that will be given. This is done by looking through all the sums that could happen, and summing the rating that we already found for this new state, multiplied by the probability of rolling that sum.
    */

    fn calculate_normal_rating(&mut self, player_score: u32, opponent_score: u32, dice_rolled: u32) -> f64 {
        if dice_rolled == 0 {
            return 1.0 - self.get_state(opponent_score, player_score, true).get_best_rating();
        }
        let mut rating: f64 = 0.0;
        for sum_dice in dice_rolled..=SIDES*dice_rolled {
            if player_score + sum_dice > MAX_SCORE { continue; }
            else {
                let this_probability: f64 = pmf(sum_dice,dice_rolled,SIDES);
                let this_rating: f64 = 1.0 - self.get_state(opponent_score, player_score + sum_dice, false).get_best_rating(); // this should always be already found
                rating += this_probability * this_rating;
            }
        }
        return rating;
    }
    pub fn calculate_optimal_normal_action(&mut self, player_score: u32, opponent_score: u32) -> State {
        let mut optimal_roll: u32 = 0; let mut optimal_rating: f64 = 0.0;
        // max_reasonable n still need tweaking
        let max_reasonable_n = 2 * (MAX_SCORE - player_score) / (SIDES + 1) + 1;
        for dice_rolled in 0..=max_reasonable_n {
            let new_rating = self.calculate_normal_rating(player_score, opponent_score, dice_rolled);
            if new_rating > optimal_rating {
                optimal_roll = dice_rolled;
                optimal_rating = new_rating;
            }
        }
        return State {
            best_n: optimal_roll,
            best_rating: optimal_rating
        };
    }
    pub fn apply_optimal_normal_actions(&mut self) {
        for order in 0..=2*MAX_SCORE {
            for place in 0..=min(order, 2*MAX_SCORE-order) {
                // calculate the player and opponet score for this order and place
                let temp1: u32 = if order < MAX_SCORE { MAX_SCORE - order } else {0};
                let temp2: u32 = if order > MAX_SCORE { order - MAX_SCORE } else {0};
                let player_score: u32 = temp1 + place;
                let opponent_score: u32 = MAX_SCORE - place - temp2;
                // set normal board states to optimal n, p
                let result: State = self.calculate_optimal_normal_action(player_score, opponent_score);
                self.set_state(player_score, opponent_score, false, result);
            }
        }
    }

    // ----- DISPLAY / WRITE DATA ----- //

    pub fn display_terminal_states(&mut self) {
        for i in 0..=MAX_SCORE {
            let f: [f64; (MAX_SCORE+1) as usize] = self.get_terminal_states()[i as usize].map(|mut item| item.get_best_rating());
            println!("{:?}", f)
        }
    }
    pub fn display_normal_states(&mut self) {
        for i in 0..=MAX_SCORE {
            let f: [f64; (MAX_SCORE+1) as usize] = self.get_normal_states()[i as usize].map(|mut item| item.get_best_rating());
            println!("{:?}", f)
        }
    }
    pub fn write_states(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal_wtr = Writer::from_path("src/results/terminal_states.csv")?;
        let mut normal_wtr = Writer::from_path("src/results/normal_states.csv")?;
        // add header
        terminal_wtr.serialize(("player_score", "opponent_score", "best_n", "best_rating"))?;
        normal_wtr.serialize(("player_score", "opponent_score", "best_n", "best_rating"))?;
        // add states
        for player_score in 0..=MAX_SCORE {
            for opponent_score in 0..=MAX_SCORE {
                // get the state
                let mut terminal_state: State = self.get_state(player_score, opponent_score, true);
                let mut normal_state: State = self.get_state(player_score, opponent_score, false);
                // write serialized state to csv
                terminal_wtr.serialize((player_score, opponent_score, terminal_state.get_best_n(), terminal_state.get_best_rating()))?;
                normal_wtr.serialize((player_score, opponent_score, normal_state.get_best_n(), normal_state.get_best_rating()))?;
            }
        }
        terminal_wtr.flush()?;
        normal_wtr.flush()?;
        Ok(())
    }
}
