import cv2
import numpy as np

def detect_arrow_direction(frame):
    gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
    edges = cv2.Canny(gray, 50, 150, apertureSize=3)

    lines = cv2.HoughLines(edges, 1, np.pi/180, 20)

    left = 0
    right = 0

    for line in lines:
        rho, theta = line[0]
        if ((np.round(theta, 2)) >= 1.0 and (np.round(theta, 2)) <= 1.1) or ((np.round(theta, 2)) >= 2.0 and (np.round(theta, 2)) <= 2.1):
            if (rho >= 20 and rho <= 30):
                left += 1
            elif (rho >= -73 and rho <= -57):
                right += 1

    if left >= 1:
        return "left"
    elif right >= 1:
        return "right"
    else:
        return "no arrow"

def main():
    video = cv2.VideoCapture("arrow.mp4")

    while True:
        ret, frame = video.read()
        if not ret:
            break
        
        frame = cv2.resize(frame, (640, 360))
        
        # Detect arrow direction
        direction = detect_arrow_direction(frame)
        
        # Overlay arrow direction on the frame
        arrow_frame = frame.copy()
        cv2.putText(arrow_frame, direction, (10, 30), cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 0, 255), 2)
        
        # Display both the original video and the frame with arrow direction
        cv2.imshow('Original Video', frame)
        cv2.imshow('Arrow Detection', arrow_frame)

    video.release()
    cv2.destroyAllWindows()

if __name__ == '__main__':
    main()