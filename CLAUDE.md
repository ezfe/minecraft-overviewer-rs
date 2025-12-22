The goal here is to re-implement the Minecraft Overviewer in Rust.

The original source code is in Minecraft-Overviewer folder.

The new code should go in src folder.

Dependencies should be added with `cargo add ...`

Ask the operator if you don't know how to accomplish a task.

---

## MINECRAFT OVERVIEWER - ARCHITECTURE SUMMARY

### What It Does
Renders 3D Minecraft worlds into interactive 2D isometric web maps (like Google Maps for Minecraft). Outputs static HTML + PNG tiles viewable in a web browser using Leaflet.js.

### High-Level Pipeline
1. **Read Minecraft world** → Parse NBT format, read region files (*.mca)
2. **Load/generate textures** → Extract from Minecraft JAR, transform to 24x24 isometric cubes
3. **Render chunks** → Layer-by-layer rendering with lighting calculations
4. **Generate tile pyramid** → Quadtree structure of 384x384 PNG tiles at multiple zoom levels
5. **Output web interface** → Static HTML/JS/CSS + PNG tiles

### Key File Formats

#### NBT (Named Binary Tag)
- Binary format, big-endian, tag-based
- Used for: level.dat, chunk data, player data
- Types: byte, short, int, long, float, double, string, byte_array, list, compound, int_array, long_array
- Often gzip or zlib compressed

#### Region Files (*.mca - Anvil format)
**Structure:**
```
[0-4095]      Location table (1024 × 4-byte big-endian ints)
[4096-8191]   Timestamp table (1024 × 4-byte big-endian ints)
[8192+]       Chunk data (variable length)
```

**Reading chunks:**
1. Read location/timestamp tables (uncompressed)
2. Calculate chunk index: `(x % 32) + (z % 32) * 32`
3. Parse location entry:
   - Offset = `(location >> 8) * 4096` bytes
   - Sectors = `location & 0xFF`
4. Seek to offset, read 5-byte header:
   - 4 bytes: data length (big-endian u32)
   - 1 byte: compression type (1=gzip, 2=zlib/deflate - almost always 2)
5. Read `(data_length - 1)` bytes of compressed data
6. Decompress with zlib
7. Parse decompressed bytes as NBT

**Important:** Region files are NOT compressed as a whole. Only individual chunks within them are zlib-compressed.

#### Chunk Data Structure (post-1.13)
- 16×16 blocks horizontal, variable height (24+ sections)
- Each section: 16×16×16 blocks
- Blocks stored with palette compression
- Contains: block states, biomes, lighting data

### Core Python Components

**Data Layer (overviewer_core/):**
- `nbt.py` - NBT parser, region file reader (MCRFileReader class)
- `world.py` - World/RegionSet classes, chunk access interface
- `biome.py` - Biome color data

**Rendering Layer:**
- `textures.py` - Texture loading, affine transformations, sprite generation
- `tileset.py` - TileSet class manages rendering world+rendermode, quadtree structure
- `rendermodes.py` - RenderPrimitive classes (lighting, cave, overlays, etc.)
- `src/*.c` - C extensions for performance-critical rendering

**Orchestration:**
- `overviewer.py` - Main entry point, CLI parsing
- `dispatcher.py` - Multiprocess job distribution
- `assetmanager.py` - Generate JavaScript config, web assets

### Rendering Details

**Isometric Projection:**
- World coords (X,Y,Z) → 2D via: `col = X + Z`, `row = Z - X`
- Block sprites: 24×24 pixels
- Affine transformations: rotate 45°, scale, shear
- Layer-by-layer rendering (Y bottom-to-top), back-to-front (painter's algorithm)

**Lighting:**
- Two values per block: BlockLight (0-15), SkyLight (0-15)
- Lighting coefficient: `c = 0.8^(15 - min(block_light, sky_light))`
- Smooth lighting: per-vertex averaging from 8 surrounding blocks
- Night mode: attenuate skylight

**Quadtree Tiles:**
- High-zoom tiles: rendered from chunks (384×384 px)
- Lower-zoom tiles: composed from 4 higher-zoom tiles (downscaled)
- Structure: zoom 0 = 1 tile, zoom N = 4^N tiles
- Each tile covers ~8 chunks vertically

**Caching:**
- Track .mca file modification times
- Only re-render affected tiles
- LRU cache for chunk data in memory

### Implementation Priorities for Rust

**Phase 1 - Foundation (Current):**
- ✓ NBT parser
- ✓ Region file reader
- World data structures
- File system utilities

**Phase 2 - Configuration:**
- CLI argument parsing
- Config file system
- Logging

**Phase 3 - Graphics:**
- Texture loading from Minecraft JAR
- Affine transformations
- Block sprite generation
- Image compositing

**Phase 4 - Rendering:**
- Block data extraction & palette decompression
- Render primitive framework
- Base rendering mode
- Lighting algorithms

**Phase 5 - Tiling:**
- Quadtree structure
- Tile generation pipeline
- Zoom level composition

**Phase 6 - Optimization:**
- Caching system
- Parallelization (rayon)
- Memory optimization

**Phase 7 - Polish:**
- Web asset generation
- Testing & compatibility
- Documentation

### Key Technical Challenges
- Handle multiple Minecraft versions (1.12-1.20+)
- Block format changes at v1.13 (palette-based)
- Biome format changes at v1.18 (2D→3D)
- Chunk height expansion (256→384 blocks)
- Performance: texture generation, image operations, chunk loading
- Multiprocessing for tile rendering
