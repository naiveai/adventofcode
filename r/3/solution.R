library(tidyverse)

num_around <- function(engine, row, col) {
    if (is.na(as.numeric(engine[row, col]))) {
        return(0)
    }

    col_start <- col
    col_end <- col

    while (col_start > 1 && !is.na(as.numeric(engine[row, col_start - 1]))) {
        col_start <- col_start - 1
    }

    while (col_end < ncol(engine) && !is.na(as.numeric(engine[row, col_end + 1]))) {
        col_end <- col_end + 1
    }

    as.numeric(str_flatten(engine[row,col_start:col_end]))
}

input <- readLines("input.txt")
engine_spec <- input |> strsplit("") |> unlist() |>
    matrix(ncol = nchar(input[1]), byrow = TRUE)
symbols_regex <- "[^\\p{L}\\d\\s\\.]"
symbol_locations <- sapply(engine_spec, \(x) grepl(symbols_regex, x, perl = TRUE)) |>
    structure(dim = dim(engine_spec)) |>
    which(arr.ind = TRUE)

engine_nums <- apply(symbol_locations, 1, \(x) list(
    c(x["row"] - 1, x["col"] - 1),
    c(x["row"], x["col"] - 1),
    c(x["row"] + 1, x["col"] - 1),
    c(x["row"] - 1, x["col"]),
    c(x["row"] + 1, x["col"]),
    c(x["row"] - 1, x["col"] + 1),
    c(x["row"], x["col"] + 1),
    c(x["row"] + 1, x["col"] + 1)
)) |> lapply(\(x) lapply(x, \(el) num_around(engine_spec, el["row"], el["col"])) |> unlist() |> unique())
sum(unlist(engine_nums))

potential_gear_locations <- sapply(engine_spec, \(x) grepl("\\*", x, perl = TRUE)) |>
    structure(dim = dim(engine_spec)) |>
    which(arr.ind = TRUE)

potential_gear_nums <- apply(potential_gear_locations, 1, \(x) list(
    c(x["row"] - 1, x["col"] - 1),
    c(x["row"], x["col"] - 1),
    c(x["row"] + 1, x["col"] - 1),
    c(x["row"] - 1, x["col"]),
    c(x["row"] + 1, x["col"]),
    c(x["row"] - 1, x["col"] + 1),
    c(x["row"], x["col"] + 1),
    c(x["row"] + 1, x["col"] + 1)
)) |> lapply(\(x) lapply(x, \(el) num_around(engine_spec, el["row"], el["col"])) |> unlist() |> unique()) |>
    lapply(\(x) x[x != 0])

gear_nums <- potential_gear_nums[unlist(lapply(potential_gear_nums, \(x) length(x) == 2))]
sum(unlist(lapply(gear_nums, prod)))
