# import RPi.GPIO as gpio
from RPi import GPIO as gpio
from time import sleep

PIN = 32

gpio.setmode(gpio.BOARD)
gpio.setup(PIN, gpio.OUT)
pin = gpio.PWM(PIN, 50000)

# Motor init
pin.start(100)
input("Press enter to confirm high pulse")
pin.ChangeDutyCycle(50)
input("Press enter to confirm low pulse")
pin.stop()
input("Press enter to confirm stopped")

try:
    pin.start(50)
    while True:
        dc = input("Input duty cycle or stop")
        if dc == 'stop':
            break
        pin.ChangeDutyCycle(int(dc))
except KeyboardInterrupt:
    pass
pin.stop()
gpio.cleanup()
