library(tidyverse)

args <- commandArgs(trailingOnly = TRUE) # requires a file path to the CSV file
greed <- data.frame(read.csv(args)) %>%
    mutate(last = if_else(last == "true", TRUE, FALSE))
terminal_states <- greed %>% filter(last == TRUE)
normal_states <- greed %>% filter(last == FALSE)

# Payoff: red (negative) -> white (zero) -> blue (positive)
payoff_colors <- scale_fill_gradient2(low = "red", mid = "white", high = "blue", midpoint = 0)

# n: white (zero) -> blue (large)
n_colors <- scale_fill_gradient(low = "white", high = "blue")

# Custom theme with larger text
greed_theme <- function() {
  theme(
    # plot.title = element_text(size = 40),
    # axis.title = element_text(size = 20),
    # axis.text = element_text(size = 20),
    # legend.title = element_text(size = 20),
    # legend.text = element_text(size = 20)
  )
}

terminal_payoffs <- ggplot(terminal_states, aes(x = active, y = queued, fill = payoff)) +
  geom_tile() +
  payoff_colors +
  labs(
      title = "Terminal Payoffs",
      x = "Score for active",
      y = "Score for queued",
      fill = "Payoff"
  ) +
  greed_theme()

normal_payoffs <- ggplot(normal_states, aes(x = active, y = queued, fill = payoff)) +
  geom_tile() +
  payoff_colors +
  labs(
      title = "Normal Payoffs",
      x = "Score for active",
      y = "Score for queued",
      fill = "Payoff"
  ) +
  greed_theme()

terminal_n <- ggplot(terminal_states, aes(x = active, y = queued, fill = n)) +
    geom_tile() +
    n_colors +
    labs(
        title = "Terminal Rolls",
        x = "Score for active",
        y = "Score for queued",
        fill = "# Dice"
    ) +
    greed_theme()

normal_n <- ggplot(normal_states, aes(x = active, y = queued, fill = n)) +
    geom_tile() +
    n_colors +
    labs(
        title = "Normal Rolls",
        x = "Score for active",
        y = "Score for queued",
        fill = "# Dice"
    ) +
    greed_theme()

save_plot <- function(plot, filename) {
    ggsave(filename, width = 12, height = 12, units = "cm", dpi = 300, plot)
}

save_plot(terminal_payoffs, "../../paper/assets/terminal_payoffs.svg")
save_plot(normal_payoffs, "../../paper/assets/normal_payoffs.svg")
save_plot(terminal_n, "../../paper/assets/terminal_n.svg")
save_plot(normal_n, "../../paper/assets/normal_n.svg")
