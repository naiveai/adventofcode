module Main (main) where

import qualified Data.Text as T
import Data.Maybe (fromJust)

main :: IO ()
main = do
    contents <- readFile "inputs/2.txt"
    let input = fromJust $ parseInput $ T.pack contents
    putStrLn $ "Part 1: " ++ (show $ sum $ map (\[me, opp] -> score me opp) input)

data RPSOption = Rock | Paper | Scissors deriving (Eq, Show)

score :: RPSOption -> RPSOption -> Int
score opponent me = baseScore + outcomeScore
    where baseScore = case me of Rock -> 1
                                 Paper -> 2
                                 Scissors -> 3
          outcomeScore = case rpsWinner opponent me of
              Nothing -> 3
              winner | winner == Just me -> 6
              _ -> 0

rpsWinner :: RPSOption -> RPSOption -> Maybe RPSOption
rpsWinner Rock Scissors = Just Rock
rpsWinner Scissors Rock = Just Rock
rpsWinner Rock Paper = Just Paper
rpsWinner Paper Rock = Just Paper
rpsWinner Paper Scissors = Just Scissors
rpsWinner Scissors Paper = Just Scissors
rpsWinner _ _ = Nothing

parseInput :: T.Text -> Maybe [[RPSOption]]
parseInput input =
    mapM ((mapM (parseRpsOption . T.unpack)) . T.splitOn (T.pack " ") . T.strip) $ T.lines input
    where
        parseRpsOption "A" = Just Rock
        parseRpsOption "B" = Just Paper
        parseRpsOption "C" = Just Scissors
        parseRpsOption "X" = Just Rock
        parseRpsOption "Y" = Just Paper
        parseRpsOption "Z" = Just Scissors
        parsseRpsOption _ = Nothing
