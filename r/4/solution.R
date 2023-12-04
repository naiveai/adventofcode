library(tidyverse)

cards <- read_table("input.txt", col_names = FALSE)

cards <- cards[-c(1, 2, 13)]
# cards <- cards[-c(1, 2, 8)]

points <- function(number) {
    if (number == 0) {
        return(0)
    }

    if (number == 1) {
        return(1)
    }

    return(2 * points(number - 1))
}

winning_range <- 1:10
card_range <- 11:35
# winning_range <- 1:5
# card_range <- 6:13

win_nums <- apply(cards, 1, \(x) length(intersect(x[winning_range], x[card_range])))

win_points <- sapply(win_nums, points)
sum(win_points)

card_nums <- rep(1, nrow(cards))
new_card_nums <- rep(1, nrow(cards))
prev_new_card_nums <- new_card_nums

while(any(new_card_nums != 0)) {
    new_card_nums <- rep(0, nrow(cards))

    for (i in seq_along(card_nums)) {
        if (is.na(i+win_nums[i])) { next }
        if (win_nums[i] == 0) { next }

        new_card_nums[(i+1):(i+win_nums[i])] <- new_card_nums[(i+1):(i+win_nums[i])] + prev_new_card_nums[i]
    }

    card_nums <- card_nums + new_card_nums
    prev_new_card_nums <- new_card_nums
}

sum(card_nums)
