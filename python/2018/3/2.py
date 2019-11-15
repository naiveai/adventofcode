"""
Amidst the chaos, you notice that exactly one claim doesn't overlap by even
a single square inch of fabric with any other claim. If you can somehow draw
attention to it, maybe the Elves will be able to make Santa's suit after all!

For example, in the claims above, only claim 3 is intact after all claims are
made.

What is the ID of the only claim that doesn't overlap?
"""
from common import claims_from_claim_strings


def find_non_overlapping_claims(claims):
    claims = list(claims)
    occupied_squares = {
        claim.id: frozenset(claim.squares_occupied())
        for claim in claims
    }
    non_overlapping_claims = []
    for claim in claims:

        # Check if this claim overlaps with any other claims
        for other_claim in claims:
            if claim.id == other_claim.id:
                continue

            overlaps_with_this = frozenset.intersection(
                occupied_squares[claim.id], occupied_squares[other_claim.id])

            if overlaps_with_this:
                # It does, no need to check further
                break
        else:
            # This only happens if no break is encountered
            non_overlapping_claims.append(claim)

        # This will only happen if a break is encountered, which means
        # we just have to move on. Placing for clarity
        continue
    return non_overlapping_claims


if __name__ == "__main__":
    with open('input.txt') as claim_strings:
        claims = claims_from_claim_strings(claim_strings)

        print(find_non_overlapping_claims(claims)[0].id)
