

def parse_number(num):
    low_bits = num & 0xFF
    high_bits = num >> 8
    print("low bits: " + str(low_bits))
    
    for bit in range(0, 8):
        mask = 1 << bit
        value = mask & high_bits
        if value:
            print("Bit " + str(bit + 8) + " is true.")

    print("All other bits are false.")

while True:
    num = int(input("Encoded number: "))
    parse_number(num)


