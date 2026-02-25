using CairoMakie
using CSV
using DataFrames
using Statistics

function visualize(file_path::String, plot::Symbol=:payoff)::Figure
    @assert (plot in [:payoff, :n])

    df = CSV.read(file_path, DataFrame)
    max = Int(floor(sqrt(nrow(df) / 2 - 1)))

    hm_normal = reshape(df[df.last .== false, plot], max+1, max+1)'
    hm_terminal = reshape(df[df.last .== true, plot], max+1, max+1)'

    type = (plot == :payoff) ? "Payoffs" : "Dice Counts"

    fig = Figure(size=(900, 500), fontsize=14, backgroundcolor=:transparent)
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

    fig_payoffs = visualize(file_path, :payoff)
    fig_rolls = visualize(file_path, :n)

    save("optimal_values.svg", fig_payoffs)
    save("optimal_policy.svg", fig_rolls)
end
