# bench.py
print("Starting Extreme Python Benchmark (5M Hailstone Sequences)...")

total_steps = 0
n = 1

while n <= 5000000:
    curr = n
    while curr > 1:
        if curr % 2 == 0:
            curr = curr // 2
        else:
            curr = (curr * 3) + 1
        total_steps += 1
    n += 1

print("Total Hailstone Steps Computed:")
print(total_steps)
