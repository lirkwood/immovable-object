import cv2 as cv
import numpy as np

cap = cv.VideoCapture("/home/linus/media/track.mp4")
writer = cv.VideoWriter(
    "/home/linus/media/lines.mp4", cv.VideoWriter_fourcc(*"mp4v"), 20, (640, 280), True
)
count = 0
while cap.isOpened():
    count += 1
    is_next, frame = cap.read()
    # frame = cv.blur(frame[100:, :], (2,2))
    frame = frame[100:, :]
    if not is_next:
        break

    center_x = frame.shape[1] // 2
    left_frame, right_frame = frame[:, :center_x], frame[:, center_x:]

    # Masking
    lower_yellow, upper_yellow = np.array([0, 150, 150]), np.array([160, 255, 255])
    lower_blue, upper_blue = np.array([134, 0, 0]), np.array([255, 128, 128])
    left_mask = cv.inRange(left_frame, lower_yellow, upper_yellow)
    right_mask = cv.inRange(right_frame, lower_blue, upper_blue)
    combined_mask = np.concatenate((left_mask, right_mask), axis=1)

    # Draw lines on frame
    lines = cv.HoughLinesP(
        combined_mask, 1, np.pi / 180, 100, minLineLength=150, maxLineGap=5
    )
    if lines is not None:
        for line in lines:
            x1, y1, x2, y2 = line[0]
            cv.line(frame, (x1, y1), (x2, y2), (0, 255, 0), 2)

    # writer.write(frame)
    cv.imwrite(f'/home/linus/images/drc/mask/mask-{count}.png', combined_mask)
    cv.imwrite(f'/home/linus/images/drc/frame/frame-{count}.png', frame)

writer.release()
