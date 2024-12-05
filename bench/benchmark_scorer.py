# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "crustyfuzz",
#     "fuzzywuzzy",
#     "matplotlib",
#     "numpy",
#     "rapidfuzz",
#     "pyqt6",
# ]
#
# [tool.uv.sources]
# crustyfuzz = { path = "../target/wheels/crustyfuzz-0.1.3-cp38-abi3-manylinux_2_34_x86_64.whl" }
# ///
from __future__ import annotations

import importlib
import random
import string
from timeit import timeit

import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np

mpl.use("qtagg")

random.seed(18)

plt.rc("font", size=13)  # controls default text sizes
plt.rc("axes", titlesize=18)  # fontsize of the axes title
plt.rc("axes", labelsize=15)  # fontsize of the x and y labels
plt.rc("xtick", labelsize=15)  # fontsize of the tick labels
plt.rc("ytick", labelsize=15)  # fontsize of the tick labels
plt.rc("legend", fontsize=15)  # legend fontsize

LIBRARIES = (
    "ratio",
    "partial_ratio",
    "token_sort_ratio",
    "token_set_ratio",
    "partial_token_sort_ratio",
    "partial_token_set_ratio",
    "QRatio",
    "WRatio",
)


def load_func(target):
    modname, funcname = target.rsplit(".", maxsplit=1)

    module = importlib.import_module(modname)
    return getattr(module, funcname)


def get_platform():
    import platform

    uname = platform.uname()
    pyver = platform.python_version()
    return f"Python {pyver} on {uname.system} ({uname.machine})"


def benchmark():
    words = [
        "".join(random.choice(string.ascii_letters + string.digits) for _ in range(10))
        for _ in range(10000)
    ]
    sample_rate = len(words) // 100
    sample = words[::sample_rate]

    total = len(words) * len(sample)

    print("System:", get_platform())
    print("Words :", len(words))
    print("Sample:", len(sample))
    print("Total : %s calls\n" % total)

    def wrap(f):
        def func():
            return len([f(x, y) for x in sample for y in words])

        return func

    fuzz = []
    rfuzz = []
    cfuzz = []

    header_list = [
        "Function",
        "CrustyFuzz",
        "RapidFuzz",
        "FuzzyWuzzy",
        "SpeedImprovementvsFuzzyWuzzy",
        "SpeedImprovementvsRapidFuzz",
    ]
    row_format = "{:>25}" * len(header_list)
    print(row_format.format(*header_list))

    for target in LIBRARIES:
        func = load_func("fuzzywuzzy.fuzz." + target)
        sec = timeit("func()", globals={"func": wrap(func)}, number=1)
        calls = total / sec
        fuzz.append(calls)

        rfunc = load_func("rapidfuzz.fuzz." + target)
        rsec = timeit("func()", globals={"func": wrap(rfunc)}, number=1)
        rcalls = total / rsec
        rfuzz.append(rcalls)

        cfunc = load_func("crustyfuzz.fuzz." + target)
        csec = timeit("func()", globals={"func": wrap(cfunc)}, number=1)
        ccalls = total / csec
        cfuzz.append(ccalls)

        print(
            row_format.format(
                target,
                f"{ccalls//1000}k",
                f"{rcalls//1000}k",
                f"{calls//1000}k",
                f"{int(100 * (sec - csec)/sec)}%",
                f"{int(100 * (rsec - csec)/rsec)}%",
            )
        )

    labels = LIBRARIES

    x = np.arange(len(labels))  # the label locations
    width = 0.25  # the width of the bars

    fig, ax = plt.subplots(figsize=(17, 10))
    rects1 = ax.bar(x - width, fuzz, width, label="FuzzyWuzzy", color="xkcd:coral")
    rects2 = ax.bar(x, rfuzz, width, label="RapidFuzz", color="#6495ED")
    rects3 = ax.bar(x + width, cfuzz, width, label="CrustyFuzz", color="maroon")

    # Add some text for labels, title and custom x-axis tick labels, etc.
    ax.set_ylabel("evaluated word pairs [inputs/s]")
    ax.set_xlabel("Scorer")
    ax.set_title(
        "The number of word pairs evaluated per second\n(the larger the better)"
    )
    ax.set_xticks(x)
    ax.set_xticklabels(labels, rotation=30)
    ax.get_yaxis().set_major_formatter(
        mpl.ticker.FuncFormatter(lambda x, _: format(int(x), ","))
    )
    ax.legend()

    def autolabel(rects):
        """Attach a text label above each bar in *rects*, displaying its height."""
        for rect in rects:
            height = rect.get_height()
            ax.annotate(
                f"{int(height):,}",
                xy=(rect.get_x() + rect.get_width() / 2, height),
                xytext=(0, 3),  # 3 points vertical offset
                textcoords="offset points",
                ha="center",
                va="bottom",
            )

    autolabel(rects1)
    autolabel(rects2)
    autolabel(rects3)
    fig.tight_layout()
    plt.show()


if __name__ == "__main__":
    benchmark()
