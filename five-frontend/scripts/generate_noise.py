#!/usr/bin/env python3
"""
Generate a simple grayscale noise texture PNG
Used for background effects in landing page components
"""

import math
import struct
import zlib
import os
import sys

def create_png_chunk(chunk_type, data):
    """Create a PNG chunk with proper CRC"""
    chunk_len = len(data)
    chunk_header = struct.pack('>I', chunk_len)
    chunk_data = chunk_type.encode() + data
    crc = zlib.crc32(chunk_data) & 0xffffffff
    chunk_crc = struct.pack('>I', crc)
    return chunk_header + chunk_data + chunk_crc

def generate_noise_png(width, height, output_path):
    """Generate a simple noise PNG"""
    print(f"Generating noise.png ({width}x{height})...")

    # Create directory if needed
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    # PNG signature
    png_signature = bytes([137, 80, 78, 71, 13, 10, 26, 10])

    # IHDR chunk (image header)
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 0, 0, 0, 0)
    ihdr_chunk = create_png_chunk('IHDR', ihdr_data)

    # Generate image data with noise
    # Each scanline: 1 filter byte + width bytes
    image_data = bytearray()

    for y in range(height):
        image_data.append(0)  # filter type for this scanline

        for x in range(width):
            # Simple noise using sine and cosine
            noise = (
                math.sin(x * 0.1) * 127 +
                math.cos(y * 0.1) * 127 +
                math.sin((x + y) * 0.05) * 20
            )
            value = max(0, min(255, int(noise + 128)))
            image_data.append(value)

    # Compress with zlib
    compressed = zlib.compress(bytes(image_data), 9)

    # IDAT chunk (image data)
    idat_chunk = create_png_chunk('IDAT', compressed)

    # IEND chunk (end of file)
    iend_chunk = create_png_chunk('IEND', b'')

    # Write PNG file
    with open(output_path, 'wb') as f:
        f.write(png_signature)
        f.write(ihdr_chunk)
        f.write(idat_chunk)
        f.write(iend_chunk)

    file_size = os.path.getsize(output_path)
    print(f"✓ Created {output_path} ({file_size} bytes)")

if __name__ == '__main__':
    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_path = os.path.join(script_dir, '../public/noise.png')

    try:
        generate_noise_png(512, 512, output_path)
        sys.exit(0)
    except Exception as e:
        print(f"✗ Error: {e}", file=sys.stderr)
        sys.exit(1)
