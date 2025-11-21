#!/usr/bin/env python3
import struct

def create_minimal_png(width, height, filename):
    """Create a minimal valid PNG file"""
    # PNG signature
    png = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk
    ihdr = struct.pack('>II', width, height)
    ihdr += b'\x08\x02\x00\x00\x00'  # 8-bit RGB, no compression
    ihdr_crc = 0x7e4b7e4b  # Placeholder CRC
    png += struct.pack('>I', len(ihdr))
    png += b'IHDR'
    png += ihdr
    png += struct.pack('>I', ihdr_crc)
    
    # IDAT chunk (minimal, just enough to be valid)
    idat = b'\x78\x9c\x63\x00\x00\x00\x02\x00\x01'  # Minimal deflate data
    idat_crc = 0x00000000  # Placeholder CRC
    png += struct.pack('>I', len(idat))
    png += b'IDAT'
    png += idat
    png += struct.pack('>I', idat_crc)
    
    # IEND chunk
    png += struct.pack('>I', 0)
    png += b'IEND'
    png += struct.pack('>I', 0xae426082)  # IEND CRC
    
    with open(filename, 'wb') as f:
        f.write(png)

create_minimal_png(32, 32, '32x32.png')
create_minimal_png(128, 128, '128x128.png')
create_minimal_png(256, 256, '128x128@2x.png')
print('Icons created')
