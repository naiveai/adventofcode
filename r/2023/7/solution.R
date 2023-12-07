library(tidyverse)

input <- read_table("input.txt", col_names = c("hand", "bid"))

calc_hand_type <- function(hand) {
  # Check for five of a kind
  if (length(unique(hand)) == 1) {
    return(7)
  }
  
  # Check for four of a kind
  if (any(table(hand) == 4)) {
    return(6)
  }
  
  # Check for full house
  if (any(table(hand) == 3) && any(table(hand) == 2)) {
    return(5)
  }
  
  # Check for three of a kind
  if (any(table(hand) == 3)) {
    return(4)
  }
  
  # Check for two pair
  if (sum(table(hand) == 2) == 2) {
    return(3)
  }
  
  # Check for one pair
  if (sum(table(hand) == 2) == 1) {
    return(2)
  }
  
  # High card
  return(1)
}

pt2_calc_hand_type <- function(hand) {
    # Find the maximum hand_type using the original hand_type function
    # when J is replaced by every single other type of card

    max(map_dbl(c("A", "K", "Q", "T", "9", "8", "7", "6", "5", "4", "3", "2"),
        \(x) calc_hand_type(str_replace(hand, "J", x))))
}

card_order <- rev(c("A", "K", "Q", "J", "T", "9", "8", "7", "6", "5", "4", "3", "2"))
pt2_card_order <- rev(c("A", "K", "Q", "T", "9", "8", "7", "6", "5", "4", "3", "2", "J"))

input_split <- input %>% mutate(hand_split = strsplit(hand, ''))

input_winnings <- input_split %>%
    mutate(hand_type = map_dbl(hand_split, calc_hand_type),
        hs1 = map_dbl(hand_split, \(x) match(x[1], card_order)),
        hs2 = map_dbl(hand_split, \(x) match(x[2], card_order)),
        hs3 = map_dbl(hand_split, \(x) match(x[3], card_order)),
        hs4 = map_dbl(hand_split, \(x) match(x[4], card_order)),
        hs5 = map_dbl(hand_split, \(x) match(x[5], card_order))) %>%
    arrange(hand_type, hs1, hs2, hs3, hs4, hs5) %>%
    mutate(winnings = row_number() * bid)

sum(input_winnings$winnings)

pt2_input_winnings <- input_split %>%
    mutate(hand_type = map_dbl(hand_split, pt2_calc_hand_type),
        hs1 = map_dbl(hand_split, \(x) match(x[1], pt2_card_order)),
        hs2 = map_dbl(hand_split, \(x) match(x[2], pt2_card_order)),
        hs3 = map_dbl(hand_split, \(x) match(x[3], pt2_card_order)),
        hs4 = map_dbl(hand_split, \(x) match(x[4], pt2_card_order)),
        hs5 = map_dbl(hand_split, \(x) match(x[5], pt2_card_order))) %>%
    arrange(hand_type, hs1, hs2, hs3, hs4, hs5) %>%
    mutate(winnings = row_number() * bid)

sum(pt2_input_winnings$winnings)
