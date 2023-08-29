from math import comb
import functools

@functools.cache
def C(l, p):
    if l <= 0:
        return 1
    return p ** (l + 1)

@functools.cache
def D(l, p):
    if l <= 0:
        return 1
    return p * D(l - 1, p) + R(l, p)

@functools.cache
def R(l, p):
    if l == 0:
        return 0
    ans = 0
    for n in range(32, 64):
        ans2 = 0
        for d in range(n - 32, 32):
            ans2 += comb(32, d) * ((p * D(l - 1, p)) ** d) * comb(32, n - d) * ((p * C(l - 1, p)) ** (n-d))
        ans += ans2 * ((1-p) ** (64 - n))
            
    return ans

@functools.cache
def B(l, p):
    return (p * D(l - 1, p)) ** 32 + R(l, p)

# print(D(1, 0.25))
# print(D(1, 0.5))
print(R(1, 0.75))
print(B(1, 0.75))
# print(D(1, 1))


print()

print(B(1, 0.25))
print(B(1, 0.5))
print(B(1, 0.75))
print(B(1, 1))

print()

print(B(2, 0.25))
print(B(2, 0.5))
print(B(2, 0.75))
print(B(2, .95))
print(B(2, 1))
