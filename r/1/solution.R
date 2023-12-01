library(tidyverse)

basic_calibration_number <- function(x) {
    only_numeric <- gsub("[^0-9.-]", "", x)
    two_digit_number_str <- paste(
        substr(only_numeric, 1, 1),
        substr(only_numeric, nchar(only_numeric), nchar(only_numeric)),
        sep=""
    )
    as.numeric(two_digit_number_str)
}

calibration_number <- function(x) {
    digits <- lapply(
        str_match_all(x, "(?=(one|two|three|four|five|six|seven|eight|nine|[0-9]))"),
        \(x) x[, 2]
    )
    digits <- str_flatten(unlist(digits))
    spelled_digits_replaced <- str_replace_all(digits, c(
        "one" = "1",
        "two" = "2",
        "three" = "3",
        "four" = "4",
        "five" = "5",
        "six" = "6",
        "seven" = "7",
        "eight" = "8",
        "nine" = "9"
    ))
    basic_calibration_number(spelled_digits_replaced)
}

input <- readLines("input.txt")

sum(basic_calibration_number(input))
sum(unlist(map(input, calibration_number)))
