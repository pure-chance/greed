include("greed.jl")
include("pmf.jl")
include("optimizer.jl")
include("visualize.jl")

using ArgParse

function parse_args(args)
    s = ArgParseSettings(
        prog = "optimize",
        description = "A policy optimizer for the game of Greed",
    )

    @add_arg_table! s begin
        "--max", "-m"
        help = "Maximum score"
        metavar = "MAX"
        arg_type = Int
        default = 100

        "--sides", "-s"
        help = "Number of sides on each die"
        metavar = "SIDES"
        arg_type = Int
        default = 6

        "--format", "-f"
        help = "Output format {stdout|csv}"
        metavar = "FORMAT"
        default = "csv"
        range_tester = x -> x in ("stdout", "csv")

        "--visualize"
        help = "Generate SVG visualizations"
        action = :store_true

        "--exact"
        help = "Use exact rational arithmetic"
        action = :store_true
    end

    return ArgParse.parse_args(args, s)
end

function main(args = ARGS)
    args = parse_args(args)

    max = args["max"]
    sides = args["sides"]
    fmt = args["format"]
    do_visualize = args["visualize"]
    T = args["exact"] ? Rational{BigInt} : Float64

    opt = optimize(T, Ruleset(max, sides))
    policy = opt.policy

    if fmt == "csv"
        mkpath("results")
        csv_filename = "results/greed_$(max)_$(sides).csv"
        try
            open(csv_filename, "w") do io
                show(io, MIME"text/csv"(), policy)
            end
            println(stderr, "Policy exported to $csv_filename")
        catch e
            println(stderr, "Failed to write CSV file: $e")
        end
    else
        show(stdout, MIME"text/csv"(), policy)
    end

    if do_visualize
        mkpath("results")
        payoff_figure, policy_figure = visualize(policy)
        save("results/optimal_policy_$(max)_$(sides).svg", policy_figure)
        save("results/optimal_payoffs_$(max)_$(sides).svg", payoff_figure)
        println(stderr, "Visualizations saved to results/")
    end
end

main()
