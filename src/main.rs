// ----- ENVIRONMENT VARIABLES ----- //

pub const MAX_SCORE: u32 = 10;
pub const SIDES: u32 = 3;

// ----- PACKAGES ----- //

mod greed;
use greed::Greed;
use std::process;

// ----- MAIN ----- //

fn main() 
{
    // generate Greed object
    let mut _greed: Greed = Greed::setup();
    // calculate optimal actions and rewards
    _greed.calculate_optimal_states();
    // write to csv
    if let Err(err) = _greed.write_states() {
        println!("{}", err);
        process::exit(1);
    }
    // give completion feedback
    println!("game parameters: MAX_SCORE = {}, SIDES = {} \nCalculation Complete \n", MAX_SCORE, SIDES);

    // DEBUG STATES
    // _greed.display_terminal_states();
    // println!("");
    // _greed.display_normal_states();

}
