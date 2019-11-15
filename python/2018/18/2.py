from common import CellTypes, LumberGrid

if __name__ == "__main__":
    with open('input.txt') as lumber_strs:
        lumber_strs = list(map(lambda s: s.strip(), lumber_strs.readlines()))

        lumber_grid = LumberGrid.from_str_list(lumber_strs)

        values = []

        # This is similar to Day 12 in that there's a loop in the input that we
        # can exploit to calculate up to this large of a number

        REQUIRED_NUM = 1000000000
        PERIODICITY = 28
        FIRST_STABLE = 627
        FIRST_STABLE_LOOPS_TO = 599

        for i in range(FIRST_STABLE + PERIODICITY):
            lumber_grid.tick()

            value =\
                (lumber_grid.type_counts[CellTypes.TREE] *
                 lumber_grid.type_counts[CellTypes.LUMBERYARD])

            values.append(value)

        print(values[(((REQUIRED_NUM - 1) - FIRST_STABLE) % PERIODICITY) +
                     FIRST_STABLE_LOOPS_TO])
