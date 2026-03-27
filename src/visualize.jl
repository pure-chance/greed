using CairoMakie
using CSV
using DataFrames

function create_heatmap(
    hm_terminal::AbstractArray,
    hm_normal::AbstractArray,
    type::Symbol,
)::Figure
    @assert type in [:payoff, :n]

    label = (type == :payoff) ? "Payoffs" : "Dice Counts"
    max = sqrt(length(hm_terminal))

    fig = Figure(size=(900, 500), fontsize=14)
    ax1 = Axis(
        fig[1, 1],
        title="$label for Normal States",
        xlabel="Score of the Active Player",
        ylabel="Score of the Queued Player",
        xticks=LinearTicks(6),
        yticks=LinearTicks(6),
    )
    ax2 = Axis(
        fig[1, 2],
        title="$label for Terminal States",
        xlabel="Score of the Active Player",
        xticks=LinearTicks(6),
        yticks=LinearTicks(6),
        yticklabelsvisible=false,
    )

    colormap =
        (type == :payoff) ? cgrad(["#e64553", "#eff1f5", "#1e66f5"], [-1.0, 0.0, 1.0]) :
        cgrad(["#eff1f5", "#209fb5", "#1e66f5"], [0.0, 0.5, 1.0])

    hm1 = heatmap!(ax1, 0:max, 0:max, hm_normal, colormap=colormap)
    heatmap!(ax2, 0:max, 0:max, hm_terminal, colormap=colormap)
    Colorbar(fig[1, 3], hm1, width=15, tickalign=0.5)

    return fig
end


function visualize(policy::Policy)::Tuple{Figure,Figure}
    max = policy.rs.max

    hm_terminal_payoffs = [action.payoff for (state, action) in policy if state.last]
    hm_normal_payoffs = [action.payoff for (state, action) in policy if !state.last]
    hm_terminal_action = [action.n for (state, action) in policy if state.last]
    hm_normal_action = [action.n for (state, action) in policy if !state.last]

    hm_terminal_payoffs = reshape(hm_terminal_payoffs, max + 1, max + 1)
    hm_normal_payoffs = reshape(hm_normal_payoffs, max + 1, max + 1)
    hm_terminal_action = reshape(hm_terminal_action, max + 1, max + 1)
    hm_normal_action = reshape(hm_normal_action, max + 1, max + 1)

    payoff_figure = create_heatmap(hm_terminal_payoffs, hm_normal_payoffs, :payoff)
    action_figure = create_heatmap(hm_terminal_action, hm_normal_action, :n)

    return payoff_figure, action_figure
end
