library(tidyverse)

args <- commandArgs(trailingOnly = TRUE)

# Check if the argument is a file path or direct CSV data
if (file.exists(args[1])) {
  # It's a file path - read the CSV file
  greed <- data.frame(read.csv(args[1])) %>%
    mutate(last = if_else(last == "true", TRUE, FALSE))
  output_dir <- "./"
} else {
  error("Invalid input: Please provide a valid CSV file path.")
}
terminal_states <- greed %>% filter(last == TRUE)
normal_states <- greed %>% filter(last == FALSE)

# Payoff: red (negative) -> white (zero) -> blue (positive)
payoff_colors <- scale_fill_gradient2(
  low = "#e64553", mid = "#eff1f5", high = "#1e66f5", midpoint = 0
)

# n: white (zero) -> blue (large)
n_colors <- scale_fill_gradient(low = "#eff1f5", high = "#1e66f5")

create_optimal_policies <- function(states, colors, title, x, y, fill) {
  ggplot(states, aes(x = active, y = queued, fill = payoff)) +
    geom_tile() +
    payoff_colors +
    labs(title = title, x = x, y = y, fill = fill) +
    theme(legend.position = "bottom") +
    coord_fixed()
}

terminal_payoffs <- create_optimal_policies(
  terminal_states,
  payoff_colors,
  title = "Terminal Payoffs",
  x = "Score for active",
  y = "Score for queued",
  fill = "Payoff"
)

normal_payoffs <- create_optimal_policies(
  normal_states,
  payoff_colors,
  title = "Normal Payoffs",
  x = "Score for active",
  y = "Score for queued",
  fill = "Payoff"
)

terminal_n <- create_optimal_policies(
  terminal_states,
  n_colors,
  title = "Terminal Rolls",
  x = "Score for active",
  y = "Score for queued",
  fill = "# Dice"
)

normal_n <- create_optimal_policies(
  normal_states,
  n_colors,
  title = "Normal Rolls",
  x = "Score for active",
  y = "Score for queued",
  fill = "# Dice"
)

save_plot <- function(plot, filename) {
  ggplot2::ggsave(filename, dpi = 1200, plot)
}

save_plot(terminal_payoffs, paste0(output_dir, "terminal_payoffs.svg"))
save_plot(normal_payoffs, paste0(output_dir, "normal_payoffs.svg"))
save_plot(terminal_n, paste0(output_dir, "terminal_n.svg"))
save_plot(normal_n, paste0(output_dir, "normal_n.svg"))
