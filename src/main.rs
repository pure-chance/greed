// ----- ENVIRONMENT VARIABLES ----- //

pub const MAX_SCORE: u32 = 100;
pub const SIDES: u32 = 6;

// ----- PACKAGES ----- //

mod greed;
use greed::Greed;
use std::process;

use std::error::Error;

// ----- MAIN ----- //

fn main() 
{
    // generate Greed object
    let mut _greed: Greed = Greed::setup();
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
