
## Work for
- rc car
- rocket
- airplane


## Interfaces
- spi (for raspberry pi)
- 2 connectors for esc/servo (pwm, 5v, gnd)
(-) 7 connectors for esc/servo (pwm, 5v, gnd) (enable power? to not use bec)
- 4 external leds
- 2 external buttons
- 2 external temperature sensors
- jtag
- dc motor controllers (either this or the servos)
(-) dc motor controllers (either this or the servos)
- exposed gnd
- exposed 3v3
- exposed 3 gpio pins
- exposed i2c

## Power
- needs to get power from 2s - 4s
- power from usb?
- provide 2A of 5v
(-) provide 2A of 5v


- we have one circuit 5v@3a for raspberry pi and stm32
- and another circuit for esc/servo 3a

## Features
Order with impedance type: JLC7628