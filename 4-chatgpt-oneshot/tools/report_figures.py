from pathlib import Path

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.patches import FancyArrowPatch, Rectangle


DESTINATION = Path(__file__).resolve().parent.parent / "figures"
INK = "#243746"
BLUE = "#d8eaf5"
PURPLE = "#e9dff1"


def canvas(width, height, xmax, ymax):
    figure, axis = plt.subplots(figsize=(width, height))
    axis.set_xlim(0, xmax)
    axis.set_ylim(0, ymax)
    axis.set_aspect("equal")
    axis.axis("off")
    figure.subplots_adjust(left=.02, right=.98, top=.98, bottom=.02)
    return figure, axis


def label(axis, x, y, text, size=12, **kwargs):
    axis.text(x, y, text, ha="center", va="center",
              fontsize=size, color=INK, **kwargs)


def arrow(axis, x1, y1, x2, y2, both=False):
    axis.add_patch(FancyArrowPatch(
        (x1, y1), (x2, y2),
        arrowstyle="<->" if both else "->",
        mutation_scale=16, linewidth=1.4, color=INK))


def save(figure, name):
    DESTINATION.mkdir(exist_ok=True)
    figure.savefig(DESTINATION / f"{name}.svg",
                   metadata={"Date": None}, facecolor="white")
    figure.savefig(DESTINATION / f"{name}.png", dpi=160,
                   facecolor="white")
    plt.close(figure)


def machine():
    figure, axis = canvas(8, 3.4, 8, 3.4)
    for x, name in ((1, "A"), (4, "D"), (7, "B")):
        label(axis, x, 3.0, name, size=18, weight="bold")
        axis.add_patch(Rectangle((x - .6, .55), 1.2, 1.95,
                                 fill=False, edgecolor=INK,
                                 linewidth=1.3))
        if name != "D":
            label(axis, x, 1.5, "empty", size=11)
    for y, value in ((2.11, "$x_0$"), (1.67, "$x_1$"),
                     (1.23, "…"), (.79, "$x_{n-1}$")):
        axis.add_patch(Rectangle((3.5, y - .17), 1, .34,
                                 facecolor=BLUE, edgecolor=INK))
        label(axis, 4, y, value)
    arrow(axis, 1.7, 2.2, 3.3, 2.2, both=True)
    arrow(axis, 4.7, 2.2, 6.3, 2.2, both=True)
    label(axis, 2.5, 2.6, "AD / DA", size=11)
    label(axis, 5.5, 2.6, "DB / BD", size=11)
    label(axis, 4, .2, "One top-card transfer across one edge = one operation.",
          size=11)
    save(figure, "3sm-machine")


def merge():
    figure, axis = canvas(12, 4.1, 13.6, 4.65)
    states = [([], [4, 1, 3, 2], []),
              ([4, 1], [3, 2], []),
              ([4, 1], [], [3, 2]),
              ([], [1, 2, 3, 4], [])]
    titles = ("Initial active cards", "First part parked",
              "Both parts parked", "Merged")
    actions = ("Top four cards of D", "sort top 2; DA DA",
               "sort top 2; DB DB", "AD BD BD AD")
    for index, state in enumerate(states):
        base = index * 3.45
        label(axis, base + 1.45, 4.25, titles[index], size=11,
              weight="bold")
        for stack_index, (name, cards) in enumerate(zip("ADB", state)):
            x = base + .18 + stack_index * 1.02
            label(axis, x + .35, 3.72, name, size=12,
                  weight="bold")
            for depth, value in enumerate(cards):
                y = 3.20 - depth * .43
                axis.add_patch(Rectangle(
                    (x, y - .17), .7, .34,
                    facecolor=BLUE if value in (1, 4) else PURPLE,
                    edgecolor=INK, linewidth=1))
                label(axis, x + .35, y, str(value), size=12)
            axis.add_patch(Rectangle(
                (x, 1.05), .7, .27, facecolor="#e7e7e7",
                edgecolor="#7b8790", hatch="///", linewidth=.7))
        label(axis, base + 1.45, .7, actions[index], size=10)
        if index < 3:
            arrow(axis, base + 2.98, 2.55, base + 3.46, 2.55)
    label(axis, 6.8, .2,
          "Gray bases are untouched cards, possibly empty. "
          "All orders are top to bottom.", size=10)
    save(figure, "3sm-merge")


if __name__ == "__main__":
    plt.rcParams.update({"font.family": "DejaVu Sans",
                         "svg.fonttype": "none",
                         "svg.hashsalt": "3sm-report"})
    machine()
    merge()
