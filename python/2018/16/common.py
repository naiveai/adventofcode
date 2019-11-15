import operator as op
import re
from collections import defaultdict

import attr


def opi(operation):
    def func(regs, a, b, c):
        regs[c] = operation(regs[a], b)
        return regs

    return func

def opr(operation):
    ifunc = opi(operation)

    def func(regs, a, b, c):
        return ifunc(regs, a, regs[b], c)

    return func

def opir(operation):
    def func(regs, a, b, c):
        regs[c] = int(operation(a, regs[b]))
        return regs

    return func

def opri(operation):
    def func(regs, a, b, c):
        regs[c] = int(operation(regs[a], b))
        return regs

    return func

def oprr(operation):
    rifunc = opri(operation)

    def func(regs, a, b, c):
        return rifunc(regs, a, regs[b], c)

    return func

def seti(regs, a, _, c):
    regs[c] = a
    return regs

def setr(regs, a, _, c):
    return seti(regs, regs[a], None, c)


ELFCODE_FUNCTIONS = {
    "addr": opr(op.add), "addi": opi(op.add),
    "mulr": opr(op.mul), "muli": opi(op.mul),
    "banr": opr(op.and_), "bani": opi(op.and_),
    "borr": opr(op.or_), "bori": opi(op.or_),
    "setr": setr, "seti": seti,
    "gtir": opir(op.gt), "gtri": opri(op.gt), "gtrr": oprr(op.gt),
    "eqir": opir(op.eq), "eqri": opri(op.eq), "eqrr": oprr(op.eq),
}


@attr.s
class Instruction:
    opcode = attr.ib(factory=int)
    params = attr.ib(factory=list)

    @classmethod
    def from_str(cls, str_):
        return cls.from_list(
            list(map(int, str_.split(" "))))

    @classmethod
    def from_list(cls, list_):
        return cls(list_[0], list_[1:])


@attr.s
class Sample:
    before = attr.ib(factory=list)
    instruction = attr.ib(factory=Instruction)
    after = attr.ib(factory=list)

    @classmethod
    def from_definition(cls, definition):
        before =\
            list(map(int,
                     re.match(r"Before: *\[(.+)\]", definition[0])
                     .groups()[0].split(",")))

        instruction = Instruction.from_str(definition[1])

        after =\
            list(map(int,
                     re.match(r"After: *\[(.+)\]", definition[2])
                     .groups()[0].split(",")))

        return cls(before, instruction, after)

    def behaves_like(self, func):
        return func(self.before[:], *self.instruction.params) == self.after
