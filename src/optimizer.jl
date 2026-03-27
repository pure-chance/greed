"""
Computes optimal strategies for Greed using dynamic programming.

The policy optimizer determines the best action (number of dice to roll) for every possible
game state by working backwards from terminal positions. This approach guarantees
mathematically optimal play.

# Algorithm Overview

## Stage 1: Terminal States

Computes optimal actions for final-round states where one player has already stood. Uses
optimization to find the dice count that maximizes win probability.

## Stage 2: Normal States

Uses dynamic programming to compute optimal actions for normal game states. States are
processed in reverse order of total score (active + queued) to ensure all future states are
already computed when needed.

# Example

```jl
using greed
ruleset = Ruleset(100, 6)
optimizer = PolicyOptimizer(ruleset)
policy = optimizer.policy
```
"""
struct PolicyOptimizer{P<:Number}
    rs::Ruleset
    pmfs::PMFLookup{P}
    policy::Policy
end

function PolicyOptimizer{P}(rs::Ruleset) where {P<:Number}
    PolicyOptimizer{P}(rs, PMFLookup{P}(rs), Policy(rs))
end

"""
Compute the complete optimal policy for the given ruleset.

Performs the full two-stage optimization: terminal states first, then norma states. After
completion, the policy can be queried for any valid gam state.
"""
function optimize(::Type{P}, rs::Ruleset)::PolicyOptimizer where {P<:Number}
    optimizer = PolicyOptimizer{P}(rs)
    optimize_terminal_states!(optimizer)
    optimize_normal_states!(optimizer)
    return optimizer
end

"""
Compute optimal actions for all terminal (last round) states.

Terminal states occur when one player has stood, triggering the final round. These states
can be optimized independently since there are no future rounds to consider.
"""
function optimize_terminal_states!(opt::PolicyOptimizer{P}) where {P<:Number}
    states =
        [State(active, queued, true) for active = 0:opt.rs.max for queued = 0:opt.rs.max]
    Threads.@threads for state in states
        opt.policy[state] = optimal_terminal_action(opt, state)
    end
end

"""
Find the optimal number of dice to roll in a terminal state.

# Algorithm

We can split the possible terminal states into 3 categories:
- If already ahead, doing nothing wins 100% of the time.
- If there is some action A where the minimum sum is greater than `queued - active` AND the
  maximum sum is less than `max_score - active`, then that action wins 100% of the time.
- Otherwise, we need to calculate the optimal action by finding the max in `range`, with
  boundaries that guarantee that the optimal action is within the range. Note that we
  prefer conservative actions if multiple actions have the same payoff.
"""
function optimal_terminal_action(
    opt::PolicyOptimizer{P},
    s::State,
)::Action{P} where {P<:Number}
    s.active > s.queued && return Action(0, one(P)) # Ahead
    opt.rs.sides * (s.queued - s.active + 1) <= opt.rs.max - s.active &&
        return Action(s.queued - s.active + 1, one(P)) # Guaranteed win

    # `range` starts at the min non-zero payoff, and ends at the maximum optimal payoff
    # (the point where the mean of the sum + active exceeds the max score).
    min_optimal_action = (s.queued - s.active) ÷ opt.rs.sides
    max_optimal_action =
        2 * max((opt.rs.max - s.active + opt.rs.sides) ÷ (opt.rs.sides + 1), 1)
    range = min_optimal_action:max_optimal_action

    possibly_optimal_actions = [Action(a, terminal_payoff(opt, s, a)) for a in range]
    return argmax(a -> a.payoff, possibly_optimal_actions)
end

"""
Calculate expected payoff for rolling a specific number of dice in a terminal state.

Computes the probability-weighted outcome considering all possible dice sums:
- Win: final score > opponent's score and ≤ max
- Tie: final score = opponent's score
- Lose: final score < opponent's score or > max (bust)
"""
function terminal_payoff(
    opt::PolicyOptimizer{P},
    s::State,
    dice_rolled::Integer,
)::P where {P<:Number}
    if dice_rolled == 0
        s.queued < s.active && return one(P)
        s.queued == s.active && return zero(P)
        s.queued > s.active && return -one(P)
    end

    sum(dice_rolled:(opt.rs.sides*dice_rolled)) do dice_total
        probability = pmf(opt.pmfs, dice_rolled, dice_total)
        payoff = if s.queued < s.active + dice_total <= opt.rs.max
            one(P)
        elseif s.queued == s.active + dice_total
            zero(P)
        else
            -one(P)
        end
        probability * payoff
    end
end

"""
Compute optimal actions for all normal (non-terminal) game states.

Uses dynamic programming with a specific ordering constraint: states must be processed in
decreasing order of (active + queued) score to ensure all reachable future states have
already been computed.

# Ordering Requirement

Normal states reference other normal states and terminal states, so they must be optimized
after terminal states and in the correct dependency order.

# Parallelization

States within each order can be computed in parallel since they don't depend on each other.
"""
function optimize_normal_states!(opt::PolicyOptimizer)
    for order in reverse(0:(2*opt.rs.max))
        Threads.@threads for place = 0:min(order, 2 * opt.rs.max - order)
            active_score = min(order, opt.rs.max) - place
            queued_score = max(order, opt.rs.max) - opt.rs.max + place
            state = State(active_score, queued_score, false)
            action = optimal_normal_action(opt, state)
            opt.policy[state] = action
        end
    end
end

"""
Find the optimal number of dice to roll in a normal (non-terminal)
state.

Considers all possible dice counts up to a mathematically derived upper bound, computing
expected payoffs that account for all possible future game states.

# Invariant

This function expects that possible future states have already been optimized.
"""
function optimal_normal_action(opt::PolicyOptimizer{P}, s::State)::Action where {P<:Number}
    # The mean is $(n)(s + 1) / 2$, thus the $n$ for which the mean next score
    # is greater than the max score is $ceil(2 * (MAX - a) / (s + 1))$. This
    # is the same as $2 * (MAX - a + s) / (s + 1)$. This is how
    # `optimal_policy_upper_bound` is calculated.
    optimal_policy_upper_bound::UInt =
        2 * (opt.rs.max - s.active + opt.rs.sides) ÷ (opt.rs.sides + 1)
    possibly_optimal_actions =
        [Action(a, normal_payoff(opt, s, a)) for a = 0:optimal_policy_upper_bound]
    return argmax(a -> a.payoff, possibly_optimal_actions)
end

"""
Calculate expected payoff for rolling a specific number of dice in a
normal state.

For each possible dice outcome, looks up the optimal payoff from the resulting state and
computes the probability-weighted expected value. Rolling 0 dice triggers the terminal
round with swapped player positions.

# Invariant

This function expects that all possible future states have already been optimized.
"""
function normal_payoff(
    opt::PolicyOptimizer{P},
    s::State,
    dice_rolled::Integer,
)::P where {P<:Number}
    if dice_rolled == 0
        terminal_state = State(s.queued, s.active, true)
        return -opt.policy[terminal_state].payoff
    end

    sum(dice_rolled:(opt.rs.sides*dice_rolled)) do dice_total
        probability = pmf(opt.pmfs, dice_rolled, dice_total)
        payoff = if s.active + dice_total <= opt.rs.max
            state = State(s.queued, s.active + dice_total, false)
            -opt.policy[state].payoff
        else
            -one(P)
        end
        probability * payoff
    end
end
