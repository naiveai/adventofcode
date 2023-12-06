library(tidyverse)

input <- readLines("input.txt")

time_dist_matrix <- read_table(I(input), col_names = FALSE)[,-1]

dist_func <- function(time_held, total_time) {
    time_held * (total_time - time_held)
}

ways_func <- function(x) {
    dists <- sapply(1:(x[1]), \(t) dist_func(t, x[1]))

    length(dists[dists > x[2]])
}

ways <- sapply(time_dist_matrix, ways_func)

prod(ways)

flattened <- apply(time_dist_matrix, 1, \(x) as.numeric(str_flatten(x)), simplify = FALSE)

ways_func(matrix(unlist(flattened), ncol = 2, byrow = FALSE))
