import random
import string
import timeit

from crustyfuzz import fuzz as fuzz_cf
from rapidfuzz import fuzz as fuzz_rf

crustyfuzz_ratio = fuzz_cf.ratio
rapidfuzz_ratio = fuzz_rf.ratio

words = [
    "".join(random.choice(string.ascii_letters + string.digits) for _ in range(10))
    for _ in range(10_000)
]
samples = words[:: len(words) // 100]


def f(scorer, samples):
    for sample in samples:
        for word in words:
            scorer(sample, word)


rf_results = timeit.repeat(lambda: f(rapidfuzz_ratio, samples), number=1, repeat=3)
cf_results = timeit.repeat(lambda: f(crustyfuzz_ratio, samples), number=1, repeat=3)

print("RapidFuzz:", sorted(rf_results))
print("CrustyFuzz:", sorted(cf_results))
