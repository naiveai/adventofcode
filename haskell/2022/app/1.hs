module Main (main) where

import qualified Data.Text as T
import Data.List (sort)
import Data.List.Split (splitOn)
import Data.Maybe (fromJust)
import Text.Read (readMaybe)

main :: IO ()
main = do
    contents <- readFile "inputs/1.txt"
    let elfLoad = fromJust $ parseInput $ T.pack contents
    let sortedLoadSums = reverse $ sort $ map sum elfLoad
    putStrLn $ "Part 1: " ++ show (head sortedLoadSums)
    putStrLn $ "Part 2: " ++ show (sum $ take 3 sortedLoadSums)

parseInput :: T.Text -> Maybe [[Int]]
parseInput input = sequence $ map (sequence . map (readMaybe . T.unpack)) $ 
    splitOn ([T.empty]) $ map T.strip $ T.lines input
