library(tidyverse)

games <- readLines("input.txt") |> str_split(";") |>
    map(function(game_str) {
        str_remove(game_str, "Game \\d: ") |>
            str_split(",") |>
            str_match_all("(\\d+) (\\w+)") |>
            lapply(function(x) {
                colors <- setNames(as.numeric(x[, 2]), x[, 3])
                # We do this to ensure there are explicit NAs for colors not
                # specified which makes things clearer overall and slightly
                # easier for part 2
                all_colors <- c(
                    red=unname(colors["red"]),
                    green=unname(colors["green"]),
                    blue=unname(colors["blue"])
                )
            })
    })

part1BagConfig <- c(red=12, green=13, blue=14)
possibleGames <- sapply(games, \(game) all(sapply(game, \(round) all(round <= part1BagConfig, na.rm = TRUE))))
sum(which(possibleGames))

minConfigs <- lapply(games, \(game) unlist(game) |> matrix(
    ncol = 3,
    byrow = TRUE,
    # We just reuse the same bag to avoid having to retype the column names, it
    # does not have any actual relevance to part 2 of course.
    dimnames=list(1:length(game), names(part1BagConfig))
) |> apply(2, \(color) max(color, na.rm = TRUE)))
sum(unlist(lapply(minConfigs, prod)))
