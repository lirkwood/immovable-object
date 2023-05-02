import cv2 as cv
import numpy as np
from collections import defaultdict

INPUT_SIZE = (640, 480)
OUTPUT_SIZE = (640, 380)
DEBUG_SIZE = (1280, 380)
TOP_CROP = INPUT_SIZE[1] - OUTPUT_SIZE[1]


def draw_lines(img, lines: list[list[list[int]]], color: tuple[int, int, int]):
    """Draws some lines (list of [[int, int, int, int]]) on an image."""
    for line in lines:
        x1, y1, x2, y2 = line[0]
        cv.line(img, (x1, y1), (x2, y2), color, 15)


SECTION_SIZE = 6
SECTION_THICKNESS = int(np.ceil(SECTION_SIZE / 2) - 1)


def get_line_coords(lines: list[list[list[int]]]) -> dict[int, int]:
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


cap = cv.VideoCapture("/home/linus/media/track.mp4")
writer = cv.VideoWriter(
    "/home/linus/media/lines.mp4", cv.VideoWriter_fourcc(*"mp4v"), 30, DEBUG_SIZE, True
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
    lower_yellow, upper_yellow = np.array([23, 20, 20]), np.array([37, 255, 255])
    lower_blue, upper_blue = np.array([105, 40, 40]), np.array([135, 255, 255])

    yellow_mask = cv.inRange(frame, lower_yellow, upper_yellow)
    blue_mask = cv.inRange(frame, lower_blue, upper_blue)
    combined_mask = np.bitwise_or(yellow_mask, blue_mask)

    yellow_lines = cv.HoughLinesP(
        yellow_mask, 1, np.pi / 180, 100, minLineLength=20, maxLineGap=20
    )
    blue_lines = cv.HoughLinesP(
        blue_mask, 1, np.pi / 180, 100, minLineLength=20, maxLineGap=20
    )

    if yellow_lines is not None:
        draw_lines(combined_mask, yellow_lines, (255, 0, 0))
        left_lines = get_line_coords(yellow_lines)
    else:
        left_lines = {}

    if blue_lines is not None:
        draw_lines(combined_mask, blue_lines, (255, 0, 0))
        right_lines = get_line_coords(blue_lines)
    else:
        right_lines = {}

    combined_mask = cv.cvtColor(combined_mask, cv.COLOR_GRAY2BGR)
    sections = make_sections(combined_mask, left_lines, right_lines)
    frame = cv.cvtColor(frame, cv.COLOR_HSV2BGR)
    writer.write(np.concatenate((frame, combined_mask), axis=1))

writer.release()
