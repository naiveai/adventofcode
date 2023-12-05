library(tidyverse)

input <- readLines("input.txt")
input_split <- by(input, cumsum(input == ""), c)

seeds <- substring(input_split[1], 8) |> strsplit(" ") |> lapply(as.numeric) |> unlist()
s2s_matrix <- read_table(I(input_split[2][[1]][-c(1, 2)]), col_names = c("soil", "seed", "range")) %>% arrange(seed)
s2f_matrix <- read_table(I(input_split[3][[1]][-c(1, 2)]), col_names = c("fertilizer", "soil", "range")) %>% arrange(soil)
f2w_matrix <- read_table(I(input_split[4][[1]][-c(1, 2)]), col_names = c("water", "fertilizer", "range")) %>% arrange(fertilizer)
w2l_matrix <- read_table(I(input_split[5][[1]][-c(1, 2)]), col_names = c("light", "water", "range")) %>% arrange(water)
l2t_matrix <- read_table(I(input_split[6][[1]][-c(1, 2)]), col_names = c("temperature", "light", "range")) %>% arrange(light)
t2h_matrix <- read_table(I(input_split[7][[1]][-c(1, 2)]), col_names = c("humidity", "temperature", "range")) %>%
    arrange(temperature)
h2loc_matrix <- read_table(I(input_split[8][[1]][-c(1, 2)]), col_names = c("location", "humidity", "range")) %>% arrange(humidity)

translate <- function(x, matrix, source=2, dest=1) {
    mapping <- findInterval(x, unlist(matrix[,source]))

    if (mapping == 0) {
        return(x)
    }

    row <- matrix[mapping,]

    if (row[source] + row$range < x) {
        return(x)
    }

    row[dest] + (x - row[source])
}

locations <- sapply(seeds, \(x) translate(x, s2s_matrix)) |>
    sapply(\(x) translate(x, s2f_matrix)) |>
    sapply(\(x) translate(x, f2w_matrix)) |>
    sapply(\(x) translate(x, w2l_matrix)) |>
    sapply(\(x) translate(x, l2t_matrix)) |>
    sapply(\(x) translate(x, t2h_matrix)) |>
    sapply(\(x) translate(x, h2loc_matrix))

min(unlist(locations))

h2loc_matrix <- h2loc_matrix %>% arrange(location)
t2h_matrix <- t2h_matrix %>% arrange(humidity)
l2t_matrix <- l2t_matrix %>% arrange(temperature)
w2l_matrix <- w2l_matrix %>% arrange(light)
f2w_matrix <- f2w_matrix %>% arrange(water)
s2f_matrix <- s2f_matrix %>% arrange(fertilizer)
s2s_matrix <- s2s_matrix %>% arrange(soil)

seed_ranges <- split(seeds, ceiling(seq_along(seeds) / 2))

i <- 0
repeat {
    s <- translate(mid, h2loc_matrix, source=1, dest=2) |>
        translate(t2h_matrix, source=1, dest=2) |>
        translate(l2t_matrix, source=1, dest=2) |>
        translate(w2l_matrix, source=1, dest=2) |>
        translate(f2w_matrix, source=1, dest=2) |>
        translate(s2f_matrix, source=1, dest=2) |>
        translate(s2s_matrix, source=1, dest=2)

    if(any(sapply(seed_ranges, \(r) s >= r[1] && s < r[1] + r[2]))) {
        print(i)
        break
    }

    i <- i + 1
}
