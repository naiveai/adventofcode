import itertools

def gen_scores(num_elves, initial):
    if len(initial) != num_elves:
        raise ValueError(
            f"There are {num_elves} elves, but only {len(initial)} elements")

    list_ = initial
    current_indexes = list(itertools.islice(itertools.count(), 0, num_elves))

    yield from list_

    while True:
        current_elements = [list_[i] for i in current_indexes]

        sum_ = sum(current_elements)
        new_elements = divmod(sum_, 10) if sum_ >= 10 else (sum_,)

        yield from new_elements

        list_.extend(new_elements)

        current_indexes = [
            (i + list_[i] + 1) % len(list_)
            for i in current_indexes
        ]
