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

#' Payoff: red (negative) -> white (zero) -> blue (positive)
payoff_colors <- scale_fill_gradient2(
  low = "red", mid = "white", high = "blue", midpoint = 0
)

#' n: white (zero) -> blue (large)
n_colors <- scale_fill_gradient(low = "white", high = "blue")

terminal_payoffs <- ggplot(
  terminal_states,
  aes(x = active, y = queued, fill = payoff)
) +
  geom_tile() +
  payoff_colors +
  labs(
    title = "Terminal Payoffs",
    x = "Score for active",
    y = "Score for queued",
    fill = "Payoff"
  )

normal_payoffs <- ggplot(
  normal_states,
  aes(x = active, y = queued, fill = payoff)
) +
  geom_tile() +
  payoff_colors +
  labs(
    title = "Normal Payoffs",
    x = "Score for active",
    y = "Score for queued",
    fill = "Payoff"
  )

terminal_n <- ggplot(terminal_states, aes(x = active, y = queued, fill = n)) +
  geom_tile() +
  n_colors +
  labs(
    title = "Terminal Rolls",
    x = "Score for active",
    y = "Score for queued",
    fill = "# Dice"
  )

normal_n <- ggplot(normal_states, aes(x = active, y = queued, fill = n)) +
  geom_tile() +
  n_colors +
  labs(
    title = "Normal Rolls",
    x = "Score for active",
    y = "Score for queued",
    fill = "# Dice"
  )

save_plot <- function(plot, filename) {
  ggplot2::ggsave(
    filename, width = 12, height = 12, units = "cm", dpi = 300, plot
  )
}

save_plot(terminal_payoffs, paste0(output_dir, "terminal_payoffs.png"))
save_plot(normal_payoffs, paste0(output_dir, "normal_payoffs.png"))
save_plot(terminal_n, paste0(output_dir, "terminal_n.png"))
save_plot(normal_n, paste0(output_dir, "normal_n.png"))
