# A Policy Optimization of Greed

## Background

Greed is a dice-based two-player game where players try to get as close to the maximum score as possible without going bust. The player whose score is higher at the end of play wins. This project implements a dynamic programming solution to determine the optimal policy for any game state.

## Game Rules

In this game, players alternate turns, choosing to roll as many dice as they like. Each die is numbered from 1 to n, (typically 6). The total rolled on a turn is added to that player's score. However, if a player's score ever exceeds the maximum threshold, (typically 100), they bust and immediately lose the game.

Play continues back and forth until one player decides to roll 0 dice, signaling the beginning of the last round. The opposing player then has one final opportunity to roll, following the same rules. Once this last turn is completed, the game ends. The player with the higher score wins; if both players have the same score, the game is declared a draw.

## Project Structure

- `/src`: Compute the optimal policy.
- `/visualize`: Create visualizations from the optimal policy CSV data.
- `/paper`: Document the mathematical theory and implementation of this problem.

## Usage

### Optimizing

```sh
# Generate csv file `greed_[max]_[sides].csv`.
cargo run --release -- optimize --max=100 --sides=6 --format=csv
# Generate a (mostly) human readable report.
cargo run --release -- optimize --max=100 --sides=6 --format=stdout
```

### Visualizing

```sh
cd visualize
# Generate svg plots of the policies and payoffs from the csv data.
# These are stored as (and at), respectively:
# - results/optimal_policy_[max]_[sides].svg
# - results/optimal_payoffs_[max]_[sides].svg
julia --project=. -e "import Pkg; Pkg.instantiate()"
julia --project=. visualize.jl ../results/greed_100_6.csv
```

![Payoffs](paper/assets/optimal_payoffs_100_6.svg)
![Policy](paper/assets/optimal_policy_100_6.svg)

## Key Findings

- **The first player has a slight advantage**: The first player has a slight advantage has an expected payoff of 0.0277. For reference, if you played 72 games with a 37-35-0 record, your expected payoff would be 0.027777…

- **Stop when you're dead**: In practical play, unless you're literally a dice-roll or 2 from the max, standing is likely to lead to a loss.

- **Strips of doom**: The most striking feature of the payoff map is the strips the appear along the normal states. If you look closely, you'll notice that they repeat in intervals equivalent to the number of `sides` on each die. This is the hint. 

  Take a look at state (94, 97, false). Even though you're behind, your expected payoff is as high as 0.1666… (1/6). This is because the opponent is screwed. You can roll 1 die. If it lands above them, then they have roll, which has a 50% chance to bust, and even if they don't, they might still be at or below your score. Rolling for the opponent is at most a payoff of 0, and likely lower. Alternatively, if you roll below them, they can either take a negative payoff roll, or stand. Then you get to make a roll with a 50% or greater chance of not busting. As it turns out, this kind of minute strategy cascades backwards to previous states, because of the probabilities of landing in states like these.
