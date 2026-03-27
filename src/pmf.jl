"""
Lookup table for dice-roll probability mass functions (PMFs).

Precomputes and stores the PMF for every dice count from 0 to an upper bound, enabling O(1)
lookup of P(sum = k | n dice, s sides). Phis is the performance-critical data structure of
the optimizer — PMF lookups happen millions of times during policy computation.

# Layout

All PMF values are stored in a single contiguous `Vector{Float64}`. A separate `offsets`
array records where each dice-count's PMF begins.

For `n` dice with `s` sides the possible sums run from `n` to `n*s`, giving `n*(s-1)+1`
values. Indexing with `(n, total)` is translated to `data[offsets[n+1] + (total-n) + 1]`.
"""
struct PMFLookup{P<:Number}
    pmfs::Vector{P}
    offsets::Vector{Int}
end

function PMFLookup{P}(rs::Ruleset)::PMFLookup{P} where {P<:Number}
    optimal_n_max = max((2 * (rs.max + rs.sides) ÷ (rs.sides + 1)), rs.max + 1)

    pmf_table = Vector{Vector{P}}(undef, optimal_n_max + 1)
    pmf_table[1] = [one(P)]
    for n = 1:optimal_n_max
        pmf_table[n+1] = convolve_uniform(pmf_table[n], rs.sides)
    end

    pmfs = reduce(vcat, pmf_table)
    offsets = [
        n + (rs.sides <= 1 ? 0 : (rs.sides - 1) * n * max(n - 1, 0) ÷ 2) for
        n = 0:optimal_n_max
    ]
    PMFLookup(pmfs, offsets)
end

"""
Convolve `pmf` with the uniform distribution on {1, …, `sides`}.

Computes the PMF of `X + U` where `X` has the given `pmf` and `U` is uniform on a single
die. Implemented as a sliding window over a running sum, giving O(|X|) time regardless of
`sides`.
"""
function convolve_uniform(pmf::Vector{P}, sides::UInt32)::Vector{P} where {P<:Number}
    convolution = zeros(P, length(pmf) + sides - 1)
    running_sum = zero(P)
    for i in eachindex(convolution)
        i <= length(pmf) && (running_sum += pmf[i])
        i > sides && (running_sum -= pmf[i-sides])
        convolution[i] = running_sum / sides
    end
    convolution
end

"""
Returns the probability `P(sum = total | n dice)`.

# Safety

Caller must ensure `n` ≤ `max_n` and `total` ≥ `n`.
"""
function pmf(lookup::PMFLookup{P}, n::Integer, total::Integer)::P where {P<:Number}
    @inbounds return lookup.pmfs[lookup.offsets[n+1]+(total-n)+1]
end
