from itertools import permutations

coins = [2, 3, 5, 7, 9]

# _ + _ * _^2 + _^3 - _ = 399
for p in permutations(coins):
    if p[0] + (p[1] * p[2]**2) + p[3]**3 - p[4] == 399:
        print(p)
        break
