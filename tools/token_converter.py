
from math import floor

# This is a simple python script to convert zork's tokens into readable words.
# It takes as input two numbers (the pair of numbers that make a token), and
# outputs the decompressed string.

def decompress(tok):
    first = floor(tok / 1600)
    tok -= first * 1600;
    
    second = floor(tok / 40)
    tok -= second * 40;

    third = tok

    first_char = num_to_char(first)
    second_char = num_to_char(second)
    third_char = num_to_char(third)
    
    return first_char + second_char + third_char

def num_to_char(num):
    if num == 0:
        return ""

    if 1 <= num and num <= 26:
        return chr(ord('A') + num - 1)

    if num == 27:
        return "-"

    if 30 <= num <= 39:
        return str(num - 30)

    raise Exception("Can't convert number: " + str(num))

while True:
    tok1 = int(input("Token 1:"))
    tok2 = int(input("Token 2:"))
    word = decompress(tok1) + decompress(tok2)
    print("word: " + word)



