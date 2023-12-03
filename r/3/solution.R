library(tidyverse)

num_around <- function(engine, row, col) {
    if (is.na(as.numeric(engine[row, col]))) {
        return(NA)
    }

    col_start <- col
    col_end <- col

    while (col_start > 1 && !is.na(as.numeric(engine[row, col_start - 1]))) {
        col_start <- col_start - 1
    }

    while (col_end < ncol(engine) && !is.na(as.numeric(engine[row, col_end + 1]))) {
        col_end <- col_end + 1
    }

    c(row, start=col_start, end=col_end)
}

adjacent <- function(x) {
    list(
        c(x["row"] - 1, x["col"] - 1),
        c(x["row"], x["col"] - 1),
        c(x["row"] + 1, x["col"] - 1),
        c(x["row"] - 1, x["col"]),
        c(x["row"] + 1, x["col"]),
        c(x["row"] - 1, x["col"] + 1),
        c(x["row"], x["col"] + 1),
        c(x["row"] + 1, x["col"] + 1)
    )
}

input <- readLines("input.txt")
engine_spec <- input |> strsplit("") |> unlist() |>
    matrix(ncol = nchar(input[1]), byrow = TRUE)
symbols_regex <- "[^\\p{L}\\d\\s\\.]"
symbol_locations <- sapply(engine_spec, \(x) grepl(symbols_regex, x, perl = TRUE)) |>
    structure(dim = dim(engine_spec)) |>
    which(arr.ind = TRUE)

engine_num_locations <- apply(symbol_locations, 1, adjacent) |>
    lapply(\(x) lapply(x, \(el) num_around(engine_spec, el["row"], el["col"]))) |> list_flatten()
engine_nums <- unique(engine_num_locations[!is.na(engine_num_locations)]) |>
    lapply(\(loc) as.numeric(str_flatten(engine_spec[loc["row"], loc["start.col"]:loc["end.col"]])))
sum(unlist(engine_nums))

potential_gear_locations <- sapply(engine_spec, \(x) grepl("\\*", x, perl = TRUE)) |>
    structure(dim = dim(engine_spec)) |>
    which(arr.ind = TRUE)

potential_gear_num_locations <- apply(potential_gear_locations, 1, adjacent) |>
    lapply(\(x) lapply(x, \(el) num_around(engine_spec, el["row"], el["col"]))) |>
    lapply(\(x) unique(x[!is.na(x)]))

gear_num_locations <- potential_gear_num_locations[lapply(potential_gear_num_locations, length) == 2]
gear_nums <- lapply(gear_num_locations, \(pair) lapply(pair,
    \(loc) as.numeric(str_flatten(engine_spec[loc["row"], loc["start.col"]:loc["end.col"]]))))

lapply(gear_nums, \(pair) unlist(pair) |> prod()) |> unlist() |> sum()
