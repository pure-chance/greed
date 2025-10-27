library(tidyverse)
library(conflicted)
conflict_prefer("filter", "dplyr")

args <- commandArgs(trailingOnly = TRUE)

# Check if the argument is a file path or direct CSV data
if (file.exists(args[1])) {
  # It's a file path - read the CSV file
  greed <- data.frame(read.csv(args[1])) %>%
    mutate(last = if_else(last == "true", TRUE, FALSE))
  output_dir <- "./"
} else {
  stop("Invalid input: Please provide a valid CSV file path.")
}
terminal_states <- greed %>% filter(last == TRUE)
normal_states <- greed %>% filter(last == FALSE)

# Payoff: red (negative) -> white (zero) -> blue (positive)
payoff_colors <- scale_fill_gradient2(
  low = "#e64553", mid = "#eff1f5", high = "#1e66f5", midpoint = 0
)

# n: white (zero) -> blue (large)
n_colors <- scale_fill_gradient(low = "#eff1f5", high = "#1e66f5")

terminal_payoffs <- ggplot(
  terminal_states,
  aes(active, queued, fill = payoff)
) +
  geom_tile() +
  payoff_colors +
  labs(
    title = "Optimal Terminal Payoffs",
    x = "Score (active)",
    y = "Score (queued)",
    fill = "Payoff"
  ) +
  theme(legend.position = "bottom") +
  coord_fixed()

normal_payoffs <- ggplot(
  normal_states,
  aes(active, queued, fill = payoff)
) +
  geom_tile() +
  payoff_colors +
  labs(
    title = "Optimal Normal Payoffs",
    x = "Score (active)",
    y = "Score (queued)",
    fill = "Payoff"
  ) +
  theme(legend.position = "bottom") +
  coord_fixed()

terminal_n <- ggplot(
  terminal_states,
  aes(x = active, y = queued, fill = n)
) +
  geom_tile() +
  n_colors +
  labs(
    title = "Optimal Terminal Actions",
    x = "Score (active)",
    y = "Score (queued)",
    fill = "n"
  ) +
  theme(legend.position = "bottom") +
  coord_fixed()

normal_n <- ggplot(
  normal_states,
  aes(x = active, y = queued, fill = n)
) +
  geom_tile() +
  n_colors +
  labs(
    title = "Optimal Normal Actions",
    x = "Score (active)",
    y = "Score (queued)",
    fill = "n"
  ) +
  theme(legend.position = "bottom") +
  coord_fixed()

save_plot <- function(plot, filename) {
  ggplot2::ggsave(filename, dpi = 1200, plot)
}

save_plot(terminal_payoffs, paste0(output_dir, "terminal_payoffs.svg"))
save_plot(normal_payoffs, paste0(output_dir, "normal_payoffs.svg"))
save_plot(terminal_n, paste0(output_dir, "terminal_n.svg"))
save_plot(normal_n, paste0(output_dir, "normal_n.svg"))
