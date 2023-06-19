import cv2
import numpy as np
import math

def detect_edges(frame):    
    # converting color to hsv
    hsv = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
    
    # Filter for yellow color
    lower_yellow = np.array([18, 94, 140], dtype="uint8")
    upper_yellow = np.array([48, 255, 255], dtype="uint8")
    yellow_mask = cv2.inRange(hsv, lower_yellow, upper_yellow)
    
    # Filter for blue color
    lower_blue = np.array([90, 100, 100], dtype="uint8")
    upper_blue = np.array([120, 255, 255], dtype="uint8")
    blue_mask = cv2.inRange(hsv, lower_blue, upper_blue)
    
    # Combine the masks
    combined_mask = cv2.bitwise_or(yellow_mask, blue_mask)
    
    # Detect edges
    edges = cv2.Canny(combined_mask, 50, 100)
    
    return edges

# defining region of interest
def region_of_interest(edges):
    height, width = edges.shape
    mask = np.zeros_like(edges)

    # cutting upper half to reduce noise and only focusing on lower half of the screen
    polygon = np.array([[
        (0, height),
        (0,  height*0.6),
        (width , height*0.6),
        (width , height),
    ]], np.int32)
    
    cv2.fillPoly(mask, polygon, 255)
    
    edges = cv2.bitwise_and(edges, mask)
    cv2.imshow("roi",edges)
    
    return edges
#performing hough transform to detect line segments
def detect_line_segments(cropped_edges):
    rho = 1  
    theta = np.pi / 180  
    min_threshold = 10
#hough transform = cv2.HoughLinesP( edged_image, rho, theta, min_threshold,min_length,maximum line gap)
 
    line_segments = cv2.HoughLinesP(cropped_edges, rho, theta, min_threshold, 
                                    np.array([]), minLineLength=5, maxLineGap=150) 

    return line_segments



# determining line segments for the frame and slope and inetercept
def average_slope_intercept(frame, line_segments):
    global stop 
    lane_lines = []
    stop = 0
    if line_segments is None:
        stop = 1
        print("no line segments detected")
        
        #stop = 1
        return lane_lines

    height, width,_ = frame.shape
    left_fit = []
    right_fit = []

    boundary = 1/3    
    #boundary = 1/2
    left_region_boundary = width * (1 - boundary)
    #left_region_boundary = width * boundary
    print('L', 	left_region_boundary)
    right_region_boundary = width * boundary
    print('R', 	right_region_boundary)
    for line_segment in line_segments:
        print(line_segment)
        for x1, y1, x2, y2 in line_segment:
            print ('x1=', x1,  'x2=',x2, 'y1=', y1, 'y2=', y2)
            if x1 == x2:
                print("skipping vertical lines (slope = infinity")
                continue
            
            fit = np.polyfit((x1, x2), (y1, y2), 1)
            slope = (y2 - y1) / (x2 - x1)
            intercept = y1 - (slope * x1)
            
            if slope < 0:
                if x1 < left_region_boundary and x2 < left_region_boundary:
                    left_fit.append((slope, intercept))
            else:
                if x1 > right_region_boundary and x2 > right_region_boundary:
                    right_fit.append((slope, intercept))

    left_fit_average = np.average(left_fit, axis=0)
    if len(left_fit) > 0:
        lane_lines.append(make_points(frame, left_fit_average))

    right_fit_average = np.average(right_fit, axis=0)
    if len(right_fit) > 0:
        lane_lines.append(make_points(frame, right_fit_average))

    return lane_lines



# 
def make_points(frame, line):
    height, width, _ = frame.shape
    
    slope, intercept = line
    
    y1 = height  # bottom of the frame
    y2 = int(y1*0.5)  # make points from middle of the frame down
    
    if slope == 0:
        slope = 0.1
        
    x1 = int((y1 - intercept) / slope)
    x2 = int((y2 - intercept) / slope)
    print('x1',x1)
    print('x2',x2)
    return [[x1, y1, x2, y2]]



# display green lines for the lanes
def display_lines(frame, lines, line_color=(0, 255, 0), line_width=15):
    line_image = np.zeros_like(frame)
    
    if lines is not None:
        for line in lines:
            for x1, y1, x2, y2 in line:
                cv2.line(line_image, (x1, y1), (x2, y2), line_color, line_width)
                
    line_image = cv2.addWeighted(frame, 0.8, line_image, 1, 1)
    
    return line_image




#display red heading line for steering
def display_heading_line(frame, steering_angle, line_color=(0, 0, 255), line_width=5 ):
    heading_image = np.zeros_like(frame)
    height, width, _ = frame.shape
    
    steering_angle_radian = steering_angle / 180.0 * math.pi
    
    x1 = int(width / 2)
    y1 = height
    x2 = int(x1 - height / 2 / math.tan(steering_angle_radian))
    y2 = int(height / 2)
    
    cv2.line(heading_image, (x1, y1), (x2, y2), line_color, line_width)
    heading_image = cv2.addWeighted(frame, 0.8, heading_image, 1, 3)
    
    return heading_image

# getting steering angle according to lane lines using tan(theta) function
def get_steering_angle(frame, lane_lines):
    
    height,width,_ = frame.shape
    
    if len(lane_lines) == 2:
        _, _, left_x2, _ = lane_lines[0][0]
        _, _, right_x2, _ = lane_lines[1][0]
        mid = int(width / 2)
        x_offset = (left_x2 + right_x2) / 2 - mid
        y_offset = int(height / 2)
        
    elif len(lane_lines) == 1:
        x1, _, x2, _ = lane_lines[0][0]
        x_offset = x2 - x1
        y_offset = int(height / 2)
        
    elif len(lane_lines) == 0:
        x_offset = 0
        y_offset = int(height / 2)
        
    angle_to_mid_radian = math.atan(x_offset / y_offset)
    angle_to_mid_deg = int(angle_to_mid_radian * 180.0 / math.pi)  
    steering_angle = angle_to_mid_deg + 90
    print('steering_angle',steering_angle)
    #kit.servo[3].angle = steering_angle
    if steering_angle > 145:
        steering_angle = 145
    elif steering_angle < 55:
        steering_angle = 55
    return steering_angle

def main():
    video = cv2.VideoCapture("qut_demo.mp4")

    while True:
        ret, frame = video.read()
        if not ret:
            break
        
        frame = cv2.resize(frame, (640, 360))
        
        # Detect edges
        edges = detect_edges(frame)
        
        # Define region of interest
        cropped_edges = region_of_interest(edges)
        
        # Detect line segments
        line_segments = detect_line_segments(cropped_edges)
        
        # Determine average slope and intercept
        lane_lines = average_slope_intercept(frame, line_segments)
        
        # Display lane lines
        line_image = display_lines(frame, lane_lines)
        
        # Calculate steering angle
        steering_angle = get_steering_angle(frame, lane_lines)
        
        # Display heading line for steering
        heading_image = display_heading_line(line_image, steering_angle)
        
        cv2.imshow("frame", heading_image)
        
        if cv2.waitKey(20) & 0xFF == ord('q'):
            break

    video.release()
    cv2.destroyAllWindows()

if __name__ == '__main__':
    main()