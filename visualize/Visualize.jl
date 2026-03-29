using CairoMakie
using CSV
using DataFrames

function visualize(file_path::String, plot::Symbol=:payoff)::Figure
    @assert (plot in [:payoff, :n])

    df = CSV.read(file_path, DataFrame)
    max = Int(floor(sqrt(nrow(df) / 2 - 1)))

    hm_normal = reshape(df[df.last.==false, plot], max + 1, max + 1)'
    hm_terminal = reshape(df[df.last.==true, plot], max + 1, max + 1)'

    type = (plot == :payoff) ? "Payoffs" : "Dice Counts"

    fig = Figure(size=(900, 500), fontsize=14)
    ax1 = Axis(
        fig[1, 1],
        title="$type for Normal States",
        xlabel="Score of the Active Player",
        ylabel="Score of the Queued Player",
        xticks=LinearTicks(6),
        yticks=LinearTicks(6),
    )
    ax2 = Axis(
        fig[1, 2],
        title="$type for Terminal States",
        xlabel="Score of the Active Player",
        xticks=LinearTicks(6),
        yticks=LinearTicks(6),
        yticklabelsvisible=false,
    )

    colormap = (plot == :payoff) ?
               cgrad(["#e64553", "#eff1f5", "#1e66f5"], [-1.0, 0.0, 1.0]) :
               cgrad(["#eff1f5", "#209fb5", "#1e66f5"], [0.0, 0.5, 1.0])

    hm1 = heatmap!(ax1, 0:max, 0:max, hm_normal, colormap=colormap)
    hm2 = heatmap!(ax2, 0:max, 0:max, hm_terminal, colormap=colormap)
    Colorbar(fig[1, 3], hm1, width=15, tickalign=0.5)

    return fig
end

if @isdefined(ARGS) && length(ARGS) > 0
    file_path = ARGS[1]
    if !endswith(file_path, ".csv")
        print("Error: Expected a CSV file, found $(file_path).\n")
        exit(1)
    end

    # The filepaths are greed_[max]_[sides].csv, so we extract [max]_[sides] by
    # slicing [7:end-4] (length("greed_") = 5, length(".csv") = 4), then
    # seperate [max] and [sides] by "_".
    max, sides = parse.(Int, split(basename(file_path)[7:end-4], "_"))

    fig_payoffs = visualize(file_path, :payoff)
    fig_rolls = visualize(file_path, :n)

    save("optimal_payoffs_$(max)_$(sides).svg", fig_payoffs)
    save("optimal_policy_$(max)_$(sides).svg", fig_rolls)
else
    print("Usage: julia visualize.jl <csv_path>\n")
    exit(1)
end
