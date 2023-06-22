from RPi import GPIO as gpio

PIN = 32

gpio.setmode(gpio.BOARD)
gpio.setup(PIN, gpio.OUT)
pin = gpio.PWM(PIN, 50000)

# Motor init
pin.start(100)
input("Confirm high")
pin.stop()
input("Confirm low")
pin.start(100)
input("Confirm high")
pin.stop()
input("Confirm low")


try:
    pin.start(10)
    while True:
        dc = input('Input duty cycle (1-100) or "stop": ')
        pin.ChangeDutyCycle(int(dc))
except ValueError:
    pass
except KeyboardInterrupt:
    pass

print("Stopping...")
pin.stop()
gpio.cleanup()
