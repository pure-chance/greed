using Test

include("greed.jl")
include("pmf.jl")
include("optimizer.jl")
include("visualize.jl")

@testset "PMFLookup" begin
    @testset "one die (d6)" begin
        L = PMFLookup{Rational{Int}}(Ruleset(1, 6))
        @test pmf(L, 1, 1) == 1 // 6
        @test pmf(L, 1, 2) == 1 // 6
        @test pmf(L, 1, 3) == 1 // 6
        @test pmf(L, 1, 4) == 1 // 6
        @test pmf(L, 1, 5) == 1 // 6
        @test pmf(L, 1, 6) == 1 // 6
    end

    @testset "two dice (2d6)" begin
        L = PMFLookup{Rational{Int}}(Ruleset(2, 6))
        @test pmf(L, 2, 2) == 1 // 36
        @test pmf(L, 2, 3) == 2 // 36
        @test pmf(L, 2, 4) == 3 // 36
        @test pmf(L, 2, 5) == 4 // 36
        @test pmf(L, 2, 6) == 5 // 36
        @test pmf(L, 2, 7) == 6 // 36
        @test pmf(L, 2, 8) == 5 // 36
        @test pmf(L, 2, 9) == 4 // 36
        @test pmf(L, 2, 10) == 3 // 36
        @test pmf(L, 2, 11) == 2 // 36
        @test pmf(L, 2, 12) == 1 // 36
    end

    @testset "three dice (3d6)" begin
        L = PMFLookup{Rational{Int}}(Ruleset(3, 6))
        @test pmf(L, 3, 3) == 1 // 216
        @test pmf(L, 3, 4) == 3 // 216
        @test pmf(L, 3, 5) == 6 // 216
        @test pmf(L, 3, 6) == 10 // 216
        @test pmf(L, 3, 7) == 15 // 216
        @test pmf(L, 3, 8) == 21 // 216
        @test pmf(L, 3, 9) == 25 // 216
        @test pmf(L, 3, 10) == 27 // 216
        @test pmf(L, 3, 11) == 27 // 216
        @test pmf(L, 3, 12) == 25 // 216
        @test pmf(L, 3, 13) == 21 // 216
        @test pmf(L, 3, 14) == 15 // 216
        @test pmf(L, 3, 15) == 10 // 216
        @test pmf(L, 3, 16) == 6 // 216
        @test pmf(L, 3, 17) == 3 // 216
        @test pmf(L, 3, 18) == 1 // 216
    end
end

@testset "PolicyOptimizer" begin
    function make_policy(
        entries::Vector{Tuple{State,Action{P}}},
        rs::Ruleset,
    )::Policy where {P<:Number}
        policy = Policy(rs)
        for (state, action) in entries
            policy[state] = action
        end
        return policy
    end

    @testset "Bellman optimality" begin
        ruleset = Ruleset(100, 6)
        optimizer = optimize(Float64, ruleset)
        policy = optimizer.policy

        for (state, action) in policy
            n_max = 2 * (ruleset.max - state.active + ruleset.sides) ÷ (ruleset.sides + 1)
            stored = action.payoff

            for n = 0:n_max
                computed = if state.last
                    terminal_payoff(optimizer, state, UInt(n))
                else
                    normal_payoff(optimizer, state, UInt(n))
                end

                @test computed <= stored + 1e-12
            end
        end
    end

    @testset "1x2 policy" begin
        ruleset = Ruleset(1, 2)
        optimizer = optimize(Rational{Int}, ruleset)
        expected = make_policy(
            [
                # terminal states
                (State(0, 0, true), Action(0, 0 // 1)),
                (State(0, 1, true), Action(1, -1 // 2)),
                (State(1, 0, true), Action(0, 1 // 1)),
                (State(1, 1, true), Action(0, 0 // 1)),
                # normal states
                (State(0, 0, false), Action(0, 0 // 1)),
                (State(0, 1, false), Action(1, -1 // 2)),
                (State(1, 0, false), Action(0, 1 // 2)),
                (State(1, 1, false), Action(0, 0 // 1)),
            ],
            ruleset,
        )
        @test optimizer.policy == expected
    end

    @testset "2x2 policy" begin
        ruleset = Ruleset(2, 2)
        optimizer = optimize(Rational{Int}, ruleset)
        expected = make_policy(
            [
                # terminal states
                (State(0, 0, true), Action(1, 1 // 1)),
                (State(0, 1, true), Action(1, 1 // 2)),
                (State(0, 2, true), Action(1, -1 // 2)),
                (State(1, 0, true), Action(0, 1 // 1)),
                (State(1, 1, true), Action(0, 0 // 1)),
                (State(1, 2, true), Action(1, -1 // 2)),
                (State(2, 0, true), Action(0, 1 // 1)),
                (State(2, 1, true), Action(0, 1 // 1)),
                (State(2, 2, true), Action(0, 0 // 1)),
                # normal states
                (State(0, 0, false), Action(1, 0 // 1)),
                (State(0, 1, false), Action(1, 1 // 4)),
                (State(0, 2, false), Action(1, -1 // 4)),
                (State(1, 0, false), Action(1, -3 // 8)),
                (State(1, 1, false), Action(0, 0 // 1)),
                (State(1, 2, false), Action(1, -1 // 2)),
                (State(2, 0, false), Action(0, 1 // 2)),
                (State(2, 1, false), Action(0, 1 // 2)),
                (State(2, 2, false), Action(0, 0 // 1)),
            ],
            ruleset,
        )
        @test optimizer.policy == expected
    end

    @testset "3x2 policy" begin
        ruleset = Ruleset(3, 2)
        optimizer = optimize(Rational{Int}, ruleset)
        expected = make_policy(
            [
                # terminal states
                (State(0, 0, true), Action(1, 1 // 1)),
                (State(0, 1, true), Action(1, 1 // 2)),
                (State(0, 2, true), Action(2, 1 // 4)),
                (State(0, 3, true), Action(2, -1 // 2)),
                (State(1, 0, true), Action(0, 1 // 1)),
                (State(1, 1, true), Action(1, 1 // 1)),
                (State(1, 2, true), Action(1, 1 // 2)),
                (State(1, 3, true), Action(1, -1 // 2)),
                (State(2, 0, true), Action(0, 1 // 1)),
                (State(2, 1, true), Action(0, 1 // 1)),
                (State(2, 2, true), Action(0, 0 // 1)),
                (State(2, 3, true), Action(1, -1 // 2)),
                (State(3, 0, true), Action(0, 1 // 1)),
                (State(3, 1, true), Action(0, 1 // 1)),
                (State(3, 2, true), Action(0, 1 // 1)),
                (State(3, 3, true), Action(0, 0 // 1)),
                # normal states
                (State(0, 0, false), Action(1, -1 // 32)),
                (State(0, 1, false), Action(1, -1 // 8)),
                (State(0, 2, false), Action(1, 3 // 16)),
                (State(0, 3, false), Action(2, -3 // 8)),
                (State(1, 0, false), Action(1, 3 // 32)),
                (State(1, 1, false), Action(1, 0 // 1)),
                (State(1, 2, false), Action(1, 1 // 4)),
                (State(1, 3, false), Action(1, -1 // 4)),
                (State(2, 0, false), Action(0, -1 // 4)),
                (State(2, 1, false), Action(1, -3 // 8)),
                (State(2, 2, false), Action(0, 0 // 1)),
                (State(2, 3, false), Action(1, -1 // 2)),
                (State(3, 0, false), Action(0, 1 // 2)),
                (State(3, 1, false), Action(0, 1 // 2)),
                (State(3, 2, false), Action(0, 1 // 2)),
                (State(3, 3, false), Action(0, 0 // 1)),
            ],
            ruleset,
        )
        @test optimizer.policy == expected
    end

    @testset "4x2 policy" begin
        ruleset = Ruleset(4, 2)
        optimizer = optimize(Rational{Int}, ruleset)
        expected = make_policy(
            [
                # terminal states
                (State(0, 0, true), Action(1, 1 // 1)),
                (State(0, 1, true), Action(2, 1 // 1)),
                (State(0, 2, true), Action(2, 3 // 4)),
                (State(0, 3, true), Action(2, 0 // 1)),
                (State(0, 4, true), Action(3, -5 // 8)),
                (State(1, 0, true), Action(0, 1 // 1)),
                (State(1, 1, true), Action(1, 1 // 1)),
                (State(1, 2, true), Action(1, 1 // 2)),
                (State(1, 3, true), Action(2, 1 // 4)),
                (State(1, 4, true), Action(2, -1 // 2)),
                (State(2, 0, true), Action(0, 1 // 1)),
                (State(2, 1, true), Action(0, 1 // 1)),
                (State(2, 2, true), Action(1, 1 // 1)),
                (State(2, 3, true), Action(1, 1 // 2)),
                (State(2, 4, true), Action(1, -1 // 2)),
                (State(3, 0, true), Action(0, 1 // 1)),
                (State(3, 1, true), Action(0, 1 // 1)),
                (State(3, 2, true), Action(0, 1 // 1)),
                (State(3, 3, true), Action(0, 0 // 1)),
                (State(3, 4, true), Action(1, -1 // 2)),
                (State(4, 0, true), Action(0, 1 // 1)),
                (State(4, 1, true), Action(0, 1 // 1)),
                (State(4, 2, true), Action(0, 1 // 1)),
                (State(4, 3, true), Action(0, 1 // 1)),
                (State(4, 4, true), Action(0, 0 // 1)),
                # normal states
                (State(0, 0, false), Action(1, -1 // 64)),
                (State(0, 1, false), Action(1, 5 // 64)),
                (State(0, 2, false), Action(1, -3 // 64)),
                (State(0, 3, false), Action(1, 5 // 16)),
                (State(0, 4, false), Action(2, -3 // 8)),
                (State(1, 0, false), Action(1, -17 // 128)),
                (State(1, 1, false), Action(1, -1 // 32)),
                (State(1, 2, false), Action(1, -1 // 8)),
                (State(1, 3, false), Action(1, 3 // 16)),
                (State(1, 4, false), Action(2, -3 // 8)),
                (State(2, 0, false), Action(1, 1 // 32)),
                (State(2, 1, false), Action(1, 3 // 32)),
                (State(2, 2, false), Action(1, 0 // 1)),
                (State(2, 3, false), Action(1, 1 // 4)),
                (State(2, 4, false), Action(1, -1 // 4)),
                (State(3, 0, false), Action(0, 0 // 1)),
                (State(3, 1, false), Action(0, -1 // 4)),
                (State(3, 2, false), Action(1, -3 // 8)),
                (State(3, 3, false), Action(0, 0 // 1)),
                (State(3, 4, false), Action(1, -1 // 2)),
                (State(4, 0, false), Action(0, 5 // 8)),
                (State(4, 1, false), Action(0, 1 // 2)),
                (State(4, 2, false), Action(0, 1 // 2)),
                (State(4, 3, false), Action(0, 1 // 2)),
                (State(4, 4, false), Action(0, 0 // 1)),
            ],
            ruleset,
        )
        @test optimizer.policy == expected
    end
end
