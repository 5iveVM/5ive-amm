#!/usr/bin/env node

/**
 * Generate a simple grayscale noise texture PNG
 * Used for background effects in landing page components
 */

const fs = require('fs');
const path = require('path');

// Simple PNG creation for grayscale noise
// PNG signature
const PNG_SIG = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

// Helper to create PNG IHDR chunk (image header)
function createIHDR(width, height) {
  const data = Buffer.alloc(13);
  data.writeUInt32BE(width, 0);
  data.writeUInt32BE(height, 4);
  data[8] = 8;      // bit depth
  data[9] = 0;      // color type (0 = grayscale)
  data[10] = 0;     // compression method
  data[11] = 0;     // filter method
  data[12] = 0;     // interlace method
  return data;
}

// CRC calculation
function crc32(buf) {
  const CRC_TABLE = new Uint32Array(256);
  for (let i = 0; i < 256; i++) {
    let c = i;
    for (let j = 0; j < 8; j++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    CRC_TABLE[i] = c >>> 0;
  }

  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = CRC_TABLE[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

// Create chunk
function createChunk(type, data) {
  const typeBuffer = Buffer.from(type);
  const combined = Buffer.concat([typeBuffer, data]);
  const crc = crc32(combined);

  const lengthBuffer = Buffer.alloc(4);
  lengthBuffer.writeUInt32BE(data.length, 0);

  const crcBuffer = Buffer.alloc(4);
  crcBuffer.writeUInt32BE(crc, 0);

  return Buffer.concat([lengthBuffer, combined, crcBuffer]);
}

// Generate noise texture
function generateNoisePNG(width, height, outputPath) {
  console.log(`Generating noise.png (${width}x${height})...`);

  // Create raw pixel data with filter bytes
  // Allocate space for scanlines (1 filter byte + width bytes per line)
  const rawDataBuffer = Buffer.alloc((width + 1) * height);
  let offset = 0;

  // Fill with Perlin-like noise (simplified)
  for (let y = 0; y < height; y++) {
    rawDataBuffer[offset++] = 0; // filter type for this scanline

    for (let x = 0; x < width; x++) {
      // Simple pseudo-random noise using sine/cosine
      const noise =
        Math.sin(x * 0.1) * 127 +
        Math.cos(y * 0.1) * 127 +
        Math.sin((x + y) * 0.05) * 20;
      const value = Math.max(0, Math.min(255, Math.floor(noise + 128)));
      rawDataBuffer[offset++] = value;
    }
  }

  // Compress with zlib (simplified: use Node's built-in zlib)
  const zlib = require('zlib');
  const compressedData = zlib.deflateSync(rawDataBuffer);

  // Assemble PNG chunks
  const chunks = [];

  // IHDR chunk
  const ihdr = createIHDR(width, height);
  chunks.push(createChunk('IHDR', ihdr));

  // IDAT chunk (compressed image data)
  chunks.push(createChunk('IDAT', compressedData));

  // IEND chunk (marks end of file)
  chunks.push(createChunk('IEND', Buffer.alloc(0)));

  // Combine PNG signature and chunks
  const pngData = Buffer.concat([PNG_SIG, ...chunks]);

  // Write to file
  fs.writeFileSync(outputPath, pngData);
  console.log(`✓ Created ${outputPath} (${pngData.length} bytes)`);
}

// Run
const outputPath = path.join(__dirname, '../public/noise.png');
const dir = path.dirname(outputPath);

// Ensure directory exists
if (!fs.existsSync(dir)) {
  fs.mkdirSync(dir, { recursive: true });
}

generateNoisePNG(512, 512, outputPath);
