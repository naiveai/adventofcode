library(tidyverse)

input <- readLines("input.txt")
directions <- strsplit(input[1], "")[[1]]
nodes <- read_table(I(str_replace_all(input[-1], "[,\\(\\)\\=]", "")), col_names = c("start", "left", "right"))

source <- "AAA"
dest <- "ZZZ"
current <- source
steps <- 0

while (current != dest) {
    for (dir in directions) {
        if (dir == "R") {
            current <- nodes %>% filter(start == current) %>% pull(right)
        } else if (dir == "L") {
            current <- nodes %>% filter(start == current) %>% pull(left)
        }
        steps <- steps + 1
        if (current == dest) {
            break
        }
    }
}

steps

start_nodes <- nodes %>% filter(str_detect(start, "A$")) %>% pull(start)
all_steps <- numeric(length(start_nodes))

for (i in seq_along(start_nodes)) {
    current <- start_nodes[i]
    steps <- 0
    while (!str_detect(current, "Z$")) {
        for (dir in directions) {
            if (dir == "R") {
                current <- nodes %>% filter(start == current) %>% pull(right)
            } else if (dir == "L") {
                current <- nodes %>% filter(start == current) %>% pull(left)
            }
            steps <- steps + 1
            if (str_detect(current, "Z$")) {
                break
            }
        }
    }
    all_steps[i] <- steps
}

print(all_steps)
