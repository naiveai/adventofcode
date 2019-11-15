"""
As you're about to begin construction, four of the Elves offer to help. "The sun will set soon; it'll go faster if we work together." Now, you need to account for multiple people working on steps simultaneously. If multiple steps
are available, workers should still begin them in alphabetical order.

Each step takes 60 seconds plus an amount corresponding to its letter: A=1,
B=2, C=3, and so on. So, step A takes 60+1=61 seconds, while step Z takes
60+26=86 seconds. No time is required between steps.

To simplify things for the example, however, suppose you only have help from
one Elf (a total of two workers) and that each step takes 60 fewer seconds (so
that step A takes 1 second and step Z takes 26 seconds). Then, using the same
instructions as above, this is how each second would be spent:

Second   Worker 1   Worker 2   Done
   0        C          .
   1        C          .
   2        C          .
   3        A          F       C
   4        B          F       CA
   5        B          F       CA
   6        D          F       CAB
   7        D          F       CAB
   8        D          F       CAB
   9        D          .       CABF
  10        E          .       CABFD
  11        E          .       CABFD
  12        E          .       CABFD
  13        E          .       CABFD
  14        E          .       CABFD
  15        .          .       CABFDE

Each row represents one second of time. The Second column identifies how many
seconds have passed as of the beginning of that second. Each worker column
shows the step that worker is currently doing (or . if they are idle). The Done
column shows completed steps.

Note that the order of the steps has changed; this is because steps now take
time to finish and multiple workers can begin multiple steps simultaneously.

In this example, it would take 15 seconds for two workers to complete these
steps.

With 5 workers and the 60+ second step durations described above, how long will
it take to complete all of the steps?
"""
import itertools
import math
import operator as op
from collections import namedtuple
from string import ascii_uppercase

from common import build_prereqs_dictionary, build_step_dictionary


class Worker:
    def __init__(self, id_=None):
        # Id isn't necessary, just helps keep track of workers in debugging.
        self.id_ = id_
        self.task = None
        self._begin_time = None

    def assign_task(self, task, current_time_step):
        self.task = task
        self._begin_time = current_time_step

    def clear_task(self):
        self.assign_task(None, None)

    def is_available(self, current_time_step):
        return self.time_remaining(current_time_step) <= 0

    def time_remaining(self, current_time_step):
        if not self.task:
            return -math.inf

        return (self._begin_time + self._time_func()) - current_time_step

    def _time_func(self):
        return ascii_uppercase.index(self.task.upper()) + 61


def get_available_workers(workers, time_step):
    for worker in workers:
        if worker.is_available(time_step):
            yield worker


def get_possible_next_tasks(just_finished_tasks, all_done_tasks, step_dict,
                            prereq_dict):
    possible_nexts =\
        sorted(
            itertools.chain.from_iterable(
                map(lambda t: step_dict[t], just_finished_tasks)))

    for possible_next in possible_nexts:
        all_prereqs_done =\
            set.issubset(prereq_dict[possible_next], set(all_done_tasks))

        if all_prereqs_done:
            yield possible_next


def find_time_to_finish(step_dict, worker_count):
    prereq_dict = build_prereqs_dictionary(step_dict)

    step_order = []

    no_prereqs = set(step_dict.keys()) - set(prereq_dict.keys())

    # The frontier always contains tasks that either:
    # a) a worker is currently doing,
    # b) can be done if a worker is available.
    frontier = no_prereqs

    workers = [Worker(id_=i) for i in range(worker_count)]

    for time_step in itertools.count():
        if not frontier:
            # This means there's nothing left that we are doing or can do. So
            # we finished last time step
            return (time_step - 1, step_order)

        just_finished_tasks = set()
        available_workers = []

        for worker in get_available_workers(workers, time_step):
            available_workers.append(worker)
            if worker.task:
                # The worker has just finished working on something.
                just_finished_tasks.add(worker.task)

                step_order.append(worker.task)

                frontier.remove(worker.task)

                worker.clear_task()

        for possible_next in get_possible_next_tasks(
                just_finished_tasks, step_order, step_dict, prereq_dict):
            frontier.add(possible_next)

        for i, task in enumerate(sorted(frontier)[:len(available_workers)]):
            if task not in map(op.attrgetter('task'), workers):
                available_workers[i].assign_task(task, time_step)


if __name__ == "__main__":
    with open('input.txt') as step_strs:
        step_dict = build_step_dictionary(step_strs)

        print(find_time_to_finish(step_dict, worker_count=5)[0])
