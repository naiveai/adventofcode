"""
Using the samples you collected, work out the number of each opcode and execute
the test program (the second section of your puzzle input).

What value is contained in register 0 after executing the test program?
"""
import more_itertools
from common import ELFCODE_FUNCTIONS, Sample, Instruction

def find_correct_behaviours(samples):
    behaviours = [set(ELFCODE_FUNCTIONS.keys()) for _ in range(16)]

    for sample in samples:
        behaves_like = set(name for name, func in ELFCODE_FUNCTIONS.items()
                           if sample.behaves_like(func))

        behaviours[sample.instruction.opcode] &= behaves_like

    while True:
        indx = []
        unique = set()

        for i, b in enumerate(behaviours):
            if len(b) == 1:
                unique.add(list(b)[0])
                indx.append(i)

        if len(unique) == len(behaviours):
            break

        for u in unique:
            for i, b in enumerate(behaviours):
                if i not in indx:
                    b.discard(u)

    return [s.pop() for s in behaviours]


if __name__ == "__main__":
    with open('input_1.txt') as samples:
        samples = filter(None, map(lambda s: s.strip(), samples.readlines()))
        samples = list(
            map(Sample.from_definition, more_itertools.chunked(samples, 3)))

        behaviours = find_correct_behaviours(samples)

        with open('input_2.txt') as program:
            program =\
                list(map(Instruction.from_str, program.readlines()))

            regs = [0] * 4

            for instruction in program:
                regs =\
                    ELFCODE_FUNCTIONS[
                        behaviours[instruction.opcode]](regs,
                                                        *instruction.params)

            print(regs[0])
