#!/usr/bin/env python3
"""PPM (P6) -> PNG без внешних библиотек. Использование: ppm2png.py in.ppm out.png [масштаб_деления]"""
import sys, zlib, struct
def main():
    src, dst = sys.argv[1], sys.argv[2]
    div = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    d = open(src, 'rb').read()
    parts = d.split(b'\n', 3)
    w, h = map(int, parts[1].split()); px = parts[3]
    rows = []
    for y in range(0, h, div):
        r = px[y*w*3:(y+1)*w*3]
        if div > 1:
            r = b''.join(r[x*3:x*3+3] for x in range(0, w, div))
        rows.append(b'\x00' + r)
    ow, oh = (w + div - 1) // div, len(rows)
    def chunk(t, data):
        c = struct.pack('>I', len(data)) + t + data
        return c + struct.pack('>I', zlib.crc32(t + data) & 0xffffffff)
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', ow, oh, 8, 2, 0, 0, 0)) \
        + chunk(b'IDAT', zlib.compress(b''.join(rows), 6)) + chunk(b'IEND', b'')
    open(dst, 'wb').write(png)
main()
