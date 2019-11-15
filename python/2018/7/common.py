from collections import defaultdict
import re

def parse_step_str(step_str):
    parts_regex = re.compile(r"Step (?P<prereq>[A-Z]) must be finished" +
                             r" before step (?P<name>[A-Z]) can begin.")

    result = parts_regex.match(step_str)

    if not result:
        raise ValueError(
            f"String {step_str} does not look like a step string.")

    groups = result.groupdict()

    return (groups['prereq'], groups['name'])


def build_step_dictionary(step_strs):
    step_dict = defaultdict(set)

    for str_ in step_strs:
        prereq, name = parse_step_str(str_)
        step_dict[prereq].add(name)

    return step_dict


def build_prereqs_dictionary(step_dict):
    prereq_dict = defaultdict(set)

    for prereq, names in step_dict.items():
        for name in names:
            prereq_dict[name].add(prereq)

    return prereq_dict
