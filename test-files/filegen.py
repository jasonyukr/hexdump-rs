#!/bin/python3

with open('zeros.bin', 'wb') as f:
    f.write(bytes([0] * 1024 * 1024 * 1024))

with open('no-squeeze.bin', 'wb') as f:
    f.write(bytes(([0] * 16 + [1]) * 1024 * 512))
