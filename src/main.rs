// ----- ALLOW UNUSED IMPORTS (THEY'RE USED FOR TESTING) ----- //
#![allow(unused_imports)]
// ----- PACKAGES ----- //
mod greed;
use greed::Greed;
use greed::MAX_SCORE;
use greed::SIDES;
use std::process;

// ----- MAIN ----- //

fn main() 
{
    // generate Greed object
    let mut _greed: Greed = Greed::setup();
    // calculate optimal actions and rewards
    _greed.apply_optimal_terminal_actions();
    _greed.apply_optimal_normal_actions();

    // write to csv
    if let Err(err) = _greed.write_states() {
        println!("{}", err);
        process::exit(1);
    }

    println!("game parameters: MAX_SCORE = {}, SIDES = {} \nCalculation Complete \n", MAX_SCORE, SIDES);

    // DEBUG STATES
    // _greed.display_terminal_states();
    // println!("");
    // _greed.display_normal_states();

}
