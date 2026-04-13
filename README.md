# A Policy Optimization of Greed

## Background

Greed is a dice-based two-player game where players try to get as close to the maximum score as possible without going bust. The player whose score is higher at the end of play wins. This project implements a dynamic programming solution to determine the optimal policy for any game state.

## Game Rules

In this game, players alternate turns, choosing to roll as many dice as they like. Each die is numbered from 1 to n, typically 6. The total rolled on a turn is added to that player's score. However, if a player's score ever exceeds the maximum threshold, typically 100, they bust and immediately lose the game.

Play continues back and forth until one player decides to roll 0 dice, signaling the beginning of the last round. The opposing player then has one final opportunity to roll, following the same rules. Once this last turn is completed, the game ends. The player with the higher score wins; if both players have the same score, the game is declared a draw.

## Project Structure

- `/src`: Compute the optimal policy.
- `/visualize`: Create visualizations from the optimal policy CSV data.
- `/paper`: Document the mathematical theory and implementation of this problem.

## Usage

### Solving

```sh
# generates csv file `greed_[max]_[sides].csv`
cargo run --release -- --max=100 --sides=6 --format=csv
# generates a (mostly) human readable report
cargo run --release -- --max=100 --sides=6 --format=stdout
```

### Visualizing

```sh
cd visualize
# generates svg files from the csv file:
# - optimal_policy_[max]_[sides].svg
# - optimal_values_[max]_[sides].svg
julia --project=. -e "import Pkg; Pkg.instantiate()"
julia --project=. visualize.jl ../results/greed_100_6.csv
```


![Payoffs](paper/assets/optimal_payoffs_100_6.svg)
![Policy](paper/assets/optimal_policy_100_6.svg)


## Key Findings

- **The first player has a slight advantage**: The first player has a slight advantage at the start of the game, but luck will subsume any advantage that they may initially have.
- **Stopping early is risky**: Even when ahead, halting before reaching ~80–90 points gives the opponent a significant chance to catch up in a single roll.
