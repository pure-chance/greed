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

terminal_payoffs <- ggplot(terminal_states, aes(x = active, y = queued, fill = payoff)) +
    geom_tile() +
    payoff_colors +
    labs(
        title = "Terminal Payoffs",
        x = "Score for active",
        y = "Score for queued",
        fill = "Payoff"
    )

normal_payoffs <- ggplot(normal_states, aes(x = active, y = queued, fill = payoff)) +
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
        fill = "Roll count"
    )

normal_n <- ggplot(normal_states, aes(x = active, y = queued, fill = n)) +
    geom_tile() +
    n_colors +
    labs(
        title = "Normal Rolls",
        x = "Score for active",
        y = "Score for queued",
        fill = "Roll count"
    )

ggsave("terminal_payoffs.png", width = 10, height = 10, terminal_payoffs)
ggsave("normal_payoffs.png", width = 10, height = 10, normal_payoffs)
ggsave("terminal_n.png", width = 10, height = 10, terminal_n)
ggsave("normal_n.png", width = 10, height = 10, normal_n)
