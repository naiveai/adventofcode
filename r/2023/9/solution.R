library(tidyverse)

sequences <- read_table("input.txt", col_names = FALSE)

pred_compute <- function(x) {
    diffs <- list(x)

    while(any(diffs[[length(diffs)]] != 0)) {
        diffs[[length(diffs) + 1]] <- diff(diffs[[length(diffs)]])
    }

    for (di in rev(seq_along(diffs))) {
        if (di == length(diffs)) {
            next
        }

        diffs[[di]] <- c(
            head(diffs[[di]], 1) - head(diffs[[di + 1]], 1),
            diffs[[di]],
            tail(diffs[[di]], 1) + tail(diffs[[di + 1]], 1)
        )
    }

    c(head(diffs[[1]], 1), tail(diffs[[1]], 1))
}

preds <- sequences %>% rowwise() %>% mutate(pred = list(pred_compute(c_across(everything())))) %>%
    summarize(forward_pred = pred[2], backward_pred = pred[1])

sum(preds$forward_pred)
sum(preds$backward_pred)
