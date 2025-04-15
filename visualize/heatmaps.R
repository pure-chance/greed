library(ggplot2)
library(dplyr)

greed <- data.frame(read.csv("greed.csv")) %>% mutate(last = if_else(last == "true", TRUE, FALSE))
terminal_states <- greed %>% filter(last == T)
normal_states <- greed %>% filter(last == F)

terminal_ratings <- ggplot(terminal_states, aes(x = active, y = queued, fill = rating)) +
  geom_tile() +
  scale_fill_gradientn(colours = c("red", "blue")) +
  labs(
      title = "Terminal Ratings",
      x = "Score for active",
      y = "Score for queued"
  )

normal_ratings <- ggplot(normal_states, aes(x = active, y = queued, fill = rating)) +
  geom_tile() +
  scale_fill_gradientn(colours = c("red", "blue")) +
  labs(
      title = "Normal Ratings",
      x = "Score for active",
      y = "Score for queued"
  )

terminal_n <- ggplot(terminal_states, aes(x = active, y = queued, fill = n)) +
    geom_tile() +
    scale_fill_gradientn(colours = c("red", "blue")) +
    labs(
        title = "Terminal Rolls",
        x = "Score for active",
        y = "Score for queued"
    )

normal_n <- ggplot(normal_states, aes(x = active, y = queued, fill = n)) +
    geom_tile() +
    scale_fill_gradientn(colours = c("red", "blue")) +
    labs(
        title = "Normal Rolls",
        x = "Score for active",
        y = "Score for queued"
    )

ggsave("terminal_ratings.png", width = 10, height = 10, terminal_ratings)
ggsave("normal_ratings.png", width = 10, height = 10, normal_ratings)
ggsave("terminal_n.png", width = 10, height = 10, terminal_n)
ggsave("normal_n.png", width = 10, height = 10, normal_n)
