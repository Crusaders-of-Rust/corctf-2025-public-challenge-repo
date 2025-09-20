import itertools
import tqdm

def xor(x, y):
    return bytes([a ^ b for a, b in zip(x, y)])

with open('flag-enc.bmp', 'rb') as f:
    d = f.read()
    header = d[:len(d) - 1024 ** 2 * 3]
    data = d[len(d) - 1024 ** 2 * 3:]

enc_chunked = list(itertools.zip_longest(*[iter(data)]*3))
dec_chunked = []

for i, chunk in tqdm.tqdm(enumerate(enc_chunked)):
    friend = i ^ (1 << 19)
    dec_chunked.append(xor(chunk, enc_chunked[friend]))

with open('flag-dec.bmp', 'wb') as f:
    f.write(header)
    f.write(b''.join(dec_chunked))

