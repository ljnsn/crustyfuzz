# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "crustyfuzz",
#     "rapidfuzz",
# ]
#
# [tool.uv.sources]
# crustyfuzz = { path = "../target/wheels/crustyfuzz-0.1.3-cp38-abi3-manylinux_2_34_x86_64.whl" }
# ///
import random
import string
import timeit

from crustyfuzz import fuzz as fuzz_cf
from rapidfuzz import fuzz as fuzz_rf
from rapidfuzz import fuzz_py as fuzz_py_rf

crustyfuzz_ratio = fuzz_cf.ratio
rapidfuzz_ratio = fuzz_rf.ratio
rapidfuzz_py_ratio = fuzz_py_rf.ratio

words = [
    "".join(random.choice(string.ascii_letters + string.digits) for _ in range(10))
    for _ in range(10_000)
]
samples = words[:: len(words) // 100]


def f(scorer, samples):
    for sample in samples:
        for word in words:
            scorer(sample, word)


rf_results = timeit.repeat(lambda: f(rapidfuzz_ratio, samples), number=1, repeat=1)
rf_py_results = timeit.repeat(
    lambda: f(rapidfuzz_py_ratio, samples), number=1, repeat=1
)
cf_results = timeit.repeat(lambda: f(crustyfuzz_ratio, samples), number=1, repeat=1)

print("RapidFuzz:", sorted(rf_results))
print("RapidFuzzPy:", sorted(rf_py_results))
print("CrustyFuzz:", sorted(cf_results))
