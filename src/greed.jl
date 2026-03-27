"""
The ruleset for a game of Greed.

A ruleset of greed is (1) the maximum (or target) score, and (2) the number of sides on
each die, written (`max`, `sides`).

For a ruleset to be valid, `max` and `sides` must be positive. If `sides` is 0, division by
zero will occur in the solver; if `max` is 0, every roll will be a bust.

The standard ruleset is `(100, 6)`.
"""
struct Ruleset
    max::UInt32
    sides::UInt32

    function Ruleset(max::Integer, sides::Integer)
        0 < max || throw(ArgumentError("max must be positive"))
        0 < sides || throw(ArgumentError("Sides must be positive"))
        return new(UInt32(max), UInt32(sides))
    end
end

Ruleset() = Ruleset(100, 6)

"""
A game state of Greed.

A state is a triple `(active, queued, last)`, where `active` is the score of the active
player, `queued` the score of the queued player, and `last` is a flag indicating whether it
is the last turn.
"""
struct State
    active::UInt32
    queued::UInt32
    last::Bool

    function State(active::Integer, queued::Integer, last::Bool)
        0 <= active || throw(ArgumentError("active must be non-negative"))
        0 <= queued || throw(ArgumentError("queued must be non-negative"))
        return new(UInt32(active), UInt32(queued), last)
    end
end

"""
A (potentially) optimal action for a single game state.

An action is both the action itself (rolling `n` dice), and a expected payoff (`payoff`).

A payoff is a scaler in [−1, 1], where `-1.0` is certain loss, `0.0` is balanced, and `1.0`
is certain victory.
"""
struct Action{P<:Number}
    n::UInt32
    payoff::P

    function Action{P}(n::Integer, payoff::P) where {P<:Number}
        -one(P) <= payoff <= one(P) || throw(ArgumentError("payoff must be in [-1, 1]"))
        return new(UInt32(n), payoff)
    end
end

Action(n::Integer, payoff::P) where {P<:Number} = Action{P}(n, payoff)
Action() = Action(0, zero(P))

"""
A Greed policy.

A *policy* is a mapping from game states to actions. An *optimal policy* is a policy that
maximizes the expected payoff for each game state.

# Memory layout

States are packed into a contiguous `Vector{Action}` indexed by

    active + (max + 1) * queued + (max + 1)^2 * last

This keeps states that share the same `queued` and `last` values adjacent, which is
friendly to the solver's inner loops (they iterate over possible `active` outcomes for a
fixed opponent score).
"""
struct Policy
    data::Vector{Action}
    rs::Ruleset
end

Policy(rs::Ruleset) = Policy(Vector{Action}(undef, 2 * (rs.max + 1)^2), rs)

"""Return the action associated with a given `State`."""
function Base.getindex(p::Policy, s::State)::Action
    idx = s.active + (p.rs.max + 1) * s.queued + (p.rs.max + 1)^2 * s.last
    return p.data[idx+1]
end

function Base.getindex(p::Policy, idx::Integer)::Action
    return p.data[idx]
end

"""Set the action associated with a given `State`."""
function Base.setindex!(p::Policy, a::Action, s::State)
    idx = s.active + (p.rs.max + 1) * s.queued + (p.rs.max + 1)^2 * s.last
    p.data[idx+1] = a
end

"""Iterate over every `(State, Action)` pair in the table."""
function Base.iterate(p::Policy, idx=1)
    idx <= length(p.data) || return nothing

    active = (idx - 1) % (p.rs.max + 1)
    queued = (idx - 1) ÷ (p.rs.max + 1) % (p.rs.max + 1)
    last = (idx - 1) ÷ (p.rs.max + 1)^2 % 2 == 1

    state = State(active, queued, last)
    action = p.data[idx]

    return ((state, action), idx + 1)
end

Base.:(==)(p1::Policy, p2::Policy) = p1.data == p2.data && p1.rs == p2.rs

function Base.show(io::IO, policy::Policy)
    for (s, a) in policy
        println(io, "$(s) => $(a)")
    end
end

function Base.show(io::IO, ::MIME"text/csv", policy::Policy)
    println(io, "active,queued,last,n,payoff")
    for (s, a) in policy
        println(io, "$(s.active),$(s.queued),$(s.last),$(a.n),$(a.payoff)")
    end
end
