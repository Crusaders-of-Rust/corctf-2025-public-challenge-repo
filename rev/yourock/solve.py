# Credits to Clubby for the Python solver, mine was written in horrendous c++

with open('encoded.rj', 'r') as f:
    words = map(str.encode, f.read().strip().split(' '))

with open('rockyou.txt', 'rb') as f:
    ry = f.read().strip().split(b'\n')

keyword = next(words)
key = ry.index(keyword)

flag = []
for pos, word in enumerate(words):
    xored_msgchr = ry.index(word) ^ key
    flag.append(xored_msgchr)
    key = xored_msgchr ^ pos

print(bytes(flag).decode())