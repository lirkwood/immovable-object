from __future__ import annotations

import cv2 as cv
import numpy as np
from numpy.polynomial import Polynomial
from collections import defaultdict
from typing import Callable, Iterable, Optional, Sized
from scipy.interpolate import interp1d
from dataclasses import dataclass
from sys import argv as call_args


INPUT_SIZE = (640, 480)
OUTPUT_SIZE = (640, 380)
DEBUG_SIZE = (1280, 380)
TOP_CROP = INPUT_SIZE[1] - OUTPUT_SIZE[1]


def draw_lines(img, lines: Iterable[Line], color: tuple[int, int, int]):
    """Draws some lines (list of [[int, int, int, int]]) on an image."""
    for line in lines:
        cv.line(img, [line.start.x, line.start.y], [line.end.x, line.end.y], color, 5)


def draw_coords(
    img, xlist: list[int], ylist: np.ndarray[np.float64], color: tuple[int, int, int]
):
    ylist = +ylist.astype(np.int64)
    x, y, last_x, last_y = None, None, None, None
    for i in range(min(len(xlist), len(ylist))):
        if x is not None:
            last_x, last_y = x, y

        x, y = int(xlist[i - 1]), ylist[i - 1]
        if y in (np.nan, np.inf) or y < 0:
            x, y = last_x, last_y
            continue
        else:
            y = np.int64(y)

        if last_x is None:
            continue
        # cv.circle(img, (x, y), 4, (0, 0, 255), thickness=-1)
        cv.line(img, (last_x, last_y), (x, y), color, 5)


def get_x_coords(y: int, lines: list[Line], default: int, comparison) -> int:
    x_vals = []
    for line in lines:
        x = line.x_from_y(y)
        if x is None:
            x = default
        x_vals.append(x)
    return default if len(x_vals) == 0 else comparison(x_vals)


def get_y_coords(default: int, lines: list[Line], comparison) -> int:
    if len(lines) == 0:
        return default
    yvals = set()
    for line in lines:
        yvals.add(line.start.y)
        yvals.add(line.end.y)

    return comparison(yvals)


def draw_centre_line(img, left_lines: list[Line], right_lines: list[Line]):
    min_y = min(
        get_y_coords(OUTPUT_SIZE[1], left_lines, min),
        get_y_coords(OUTPUT_SIZE[1], right_lines, min),
    )
    x, last_x, last_y = None, None, None
    lines = []
    for y in range(min_y, OUTPUT_SIZE[1], 5):
        last_x, last_y = x, y
        # TODO check for left lines to the right of right lines

        if y > 100:
            ...  # break
        left, right = get_x_coords(y, left_lines, 0, max), get_x_coords(
            y, right_lines, OUTPUT_SIZE[0], min
        )

        x = int((left + right) / 2)
        if last_x is not None:
            lines.append(Line(Point(last_x, last_y), Point(x, y)))
    draw_lines(img, lines, (255, 0, 255))


SECTION_SIZE = 6
SECTION_THICKNESS = int(np.ceil(SECTION_SIZE / 2) - 1)


def get_line_coords(
    lines: list[list[list[int]]], x_selector: Callable[[Iterable[int]], int]
) -> dict[int, int]:
    """Takes a set of lines (list of 4 ints).
    Returns y values from along the line
    and a set of corresponding x values that lie on or near the lines."""
    rows = defaultdict(set)
    for line in lines:
        # iterate y coords at index 0, 2
        yvals = line[0][1::2]
        for index, y in enumerate(yvals):
            rows[5 * round(y / 5)].add(line[0][index * 2])

        if yvals[0] != yvals[1]:
            gradient = (line[0][0] - line[0][2]) / (yvals[0] - yvals[1])
        else:
            gradient = 0

        if yvals[0] < yvals[1]:
            min_y, min_y_x = yvals[0], line[0][0]
        else:
            min_y, min_y_x = yvals[1], line[0][2]

        for y in range(
            SECTION_SIZE * (round(min_y / SECTION_SIZE) + 1),
            max(yvals) - 1,
            SECTION_SIZE,
        ):
            rows[y].add(min_y_x + round(gradient * (y - min_y)))

    return {key: round(np.average(list(vals))) for key, vals in rows.items()}


def interp_lines(lines: list[Line]) -> Callable[[Iterable[int]], list[int]]:
    x, y = [], []
    for line in lines:
        x.append(line.start.x)
        x.append(line.end.x)
        y.append(line.start.y)
        y.append(line.end.y)
    return interp1d(x, y, fill_value="extrapolate")


def make_sections(img, left_lines: dict[int, int], right_lines: dict[int, int]):
    """Takes two dicts mapping y coords to x coords.
    One set of points acts as the left ends of horizontal lines.
    The other, right ends."""
    for y in range(0, OUTPUT_SIZE[1], SECTION_SIZE):
        if y in left_lines and left_lines[y]:
            left = left_lines[y]
        else:
            left = 0

        if y in right_lines and right_lines[y]:
            right = right_lines[y]
        else:
            right = OUTPUT_SIZE[0]

        cv.line(img, (left, y), (right, y), (0, 255, 0), SECTION_THICKNESS)


@dataclass(frozen=True)
class Point:
    x: int
    y: int

    def dist(self, point: Point) -> float:
        """Returns the absolute distance between these points."""
        return np.positive(self.x - point.x) + np.positive(self.y - point.y)


def ccw(a: Point, b: Point, c: Point) -> bool:
    """Returns true if points are listed in counter-clockwise order."""
    return (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)


@dataclass
class Line:
    """Describes a boundary line."""

    start: Point
    """Start x,y coords"""
    end: Point
    """End x,y coords"""

    def __init__(self, start: Point, end: Point) -> None:
        self.start = start
        self.end = end

    @classmethod
    def from_hough(cls, line: list[list[int]]) -> Line:
        return cls(Point(*line[0][:2]), Point(*line[0][2:]))

    def cross(self, line: Line) -> bool:
        """Whether these lines intersect."""
        return ccw(self.start, line.start, line.end) != ccw(
            self.end, line.start, line.end
        ) and ccw(self.start, self.end, line.start) != ccw(
            self.start, self.end, line.end
        )

    def point_dist(self, point: Point) -> np.float64:
        """Returns minimum distance from point to this line."""
        xdiff, ydiff = self.start.x - self.end.x, self.start.y - self.end.y
        magic = ((point.x - self.end.x) * xdiff + (point.y - self.end.y) * ydiff) / (
            xdiff * xdiff + ydiff * ydiff
        )

        closest: Point
        if magic < 0:
            closest = self.end
        elif magic > 1:
            closest = self.start
        else:
            closest = Point(
                int(self.end.x + magic * xdiff), int(self.end.y + magic * ydiff)
            )
        return closest.dist(point)

    @property
    def y_range(self) -> range:
        if self.start.y < self.end.y:
            y_vals = self.start.y, self.end.y
        else:
            y_vals = (
                self.end.y,
                self.start.y,
            )

        return range(*y_vals)

    @property
    def gradient(self) -> float:
        return (self.start.x - self.end.x) / ((self.start.y - self.end.y) or 0.001)

    def x_from_y(self, y: int) -> Optional[int]:
        """Returns the x coordinate for the corresponding y coord if there is one."""
        if y in self.y_range:
            return int(self.gradient * (y - self.start.y) + self.start.x)
        return None


def min_dist(line: Line, candidates: list[Line]) -> int:
    """Returns minimum distance between this line and any candidate."""
    dist = max(*OUTPUT_SIZE)
    for cand in candidates:
        if cand.cross(line):
            return 0
        dist = min(
            dist,
            *map(
                float,
                (
                    cand.point_dist(line.start),
                    cand.point_dist(line.end),
                    line.point_dist(cand.start),
                    line.point_dist(cand.end),
                ),
            ),
        )

    return np.float64(dist)


MAX_POINT_DIST = 15
"""Max num of px that points of two lines can be before they're not in a group."""


def reduce_lines(lines: Iterable[Line]) -> list[list[Line]]:
    # List of groups of lines.
    # Each sublist contains lines that are close to each other.
    groups: list[list[Line]] = []
    for line in lines:
        chosen = False
        for group in groups:
            if min_dist(line, group) < MAX_POINT_DIST:
                group.append(line)
                chosen = True
                break

        if not chosen:
            groups.append([line])

    return groups


YELLOW_BOUNDS = [np.array([23, 40, 40]), np.array([37, 255, 255])]
BLUE_BOUNDS = [np.array([105, 40, 40]), np.array([135, 255, 255])]

cap = cv.VideoCapture(call_args[1])
writer = cv.VideoWriter(
    call_args[2],
    cv.VideoWriter_fourcc(*"mp4v"),
    30,
    DEBUG_SIZE,
    True,
)
count = 0
while cap.isOpened():
    count += 1
    is_next, frame = cap.read()
    if not is_next:
        break

    frame = cv.cvtColor(frame[TOP_CROP:, :], cv.COLOR_BGR2HSV)
    center_x = frame.shape[1] // 2

    # Masking
    yellow_mask = cv.inRange(frame, *YELLOW_BOUNDS)
    blue_mask = cv.inRange(frame, *BLUE_BOUNDS)
    combined_mask = np.bitwise_or(yellow_mask, blue_mask)

    yellow_lines = cv.HoughLinesP(
        yellow_mask, 1, np.pi / 180, 100, minLineLength=20, maxLineGap=20
    )

    blue_lines = cv.HoughLinesP(
        blue_mask, 1, np.pi / 180, 100, minLineLength=20, maxLineGap=20
    )

    combined_mask = cv.cvtColor(combined_mask, cv.COLOR_GRAY2BGR)

    left_lines = []
    if yellow_lines is not None:
        max_group, max_len = None, 0
        for group in reduce_lines(map(Line.from_hough, yellow_lines)):
            if len(group) > max_len:
                max_group, max_len = group, len(group)

        if max_group is not None:
            left_lines = max_group
            draw_lines(
                combined_mask,
                max_group,
                color=(0, 255, 255),
            )

            # min_coords = {}
            # for point in {l.start for l in max_group} | {l.end for l in max_group}:
            #     if point.x in min_coords:
            #         min_coords[point.x] = min(min_coords[point.x], point.y)
            #     else:
            #         min_coords[point.x] = point.y

            # draw_coords(
            #     combined_mask,
            #     np.array([int(x) for x in min_coords.keys()]),
            #     np.array([int(y) for y in min_coords.values()]),
            #     color=(0, 255, 255),
            # )

            # herm = Polynomial.fit(
            #     np.array([int(x) for x in min_coords.keys()]),
            #     np.array([int(y) for y in min_coords.values()]),
            #     [1],
            # )
            # draw_coords(
            #     combined_mask,
            #     *herm.linspace(),
            #     color=(0, 255, 255),
            # )

    right_lines = []
    if blue_lines is not None:
        max_group, max_len = None, 0
        for group in reduce_lines(map(Line.from_hough, blue_lines)):
            if len(group) > max_len:
                max_group, max_len = group, len(group)

        if max_group is not None:
            right_lines = max_group
            draw_lines(combined_mask, max_group, (255, 0, 0))

    draw_centre_line(combined_mask, left_lines, right_lines)
    frame = cv.cvtColor(frame, cv.COLOR_HSV2BGR)
    writer.write(np.concatenate((frame, combined_mask), axis=1))

writer.release()
