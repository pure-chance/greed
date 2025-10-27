using GLMakie, GeometryBasics, CSV, DataFrames

optimal_policy = CSV.read(ARGS[1], DataFrame)

fig = Figure()

for (id, terminal) in enumerate([true, false])
    ax = Axis3(
        fig[id, 1],
        aspect=:data,
        xlabel="Active", ylabel="Queued", zlabel="Dice Rolled",
        perspectiveness=0.5
    )

    partial_policy = filter(row -> row.last == terminal, optimal_policy)
    active = partial_policy[:, :active]
    queued = partial_policy[:, :queued]
    rolls = partial_policy[:, :n]
    payoff = partial_policy[:, :payoff]

    colors = cgrad(["#e64553", "#eff1f5", "#1e66f5"], [0.0, 0.5, 1.0])
    rectMesh = Rect3f(Vec3f(-0.5, -0.5, 0), Vec3f(1, 1, 1))
    meshscatter!(
        ax, active, queued, color=payoff,
        marker=rectMesh,
        markersize=Vec3f.(1, 1, rolls[:] .+ 1e-2),
        colormap = colors,
    )
end

wait(display(fig))
