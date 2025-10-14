using CairoMakie, CSV, DataFrames

# load data
df = CSV.read(ARGS[1], DataFrame)

# process data into relevant vectors
normal = df[df.last .== false, :]
terminal = df[df.last .== true, :]

active = normal[:, :active]
queued = normal[:, :queued]

payoffs_normal = normal[:, :payoff]
payoffs_terminal = terminal[:, :payoff]

rolls_normal = normal[:, :n]
rolls_terminal = terminal[:, :n]

# build figures
fig = Figure()

payoff_colors = cgrad(["#e64553", "#eff1f5", "#1e66f5"], [0.0, 0.5, 1.0])
roll_colors = cgrad(["#eff1f5", "#1e66f5"])

axs = [Axis(fig[i, j], aspect=1, xticklabelsize=10, yticklabelsize=10) for j in 1:2, i in 1:2]


heatmap!(axs[1], active, queued, payoffs_normal, colormap=payoff_colors)
heatmap!(axs[2], active, queued, payoffs_terminal, colormap=payoff_colors)
heatmap!(axs[3], active, queued, rolls_normal, colormap=roll_colors)
heatmap!(axs[4], active, queued, rolls_terminal, colormap=roll_colors)


display(fig)
