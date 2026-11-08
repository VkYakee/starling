# HorizonVoxelCracker

**Author**: VkYakee  
**Metadata**: VkYakee VoxelMap x seedcrackerX  
**License**: MIT  

A unified Rust mod for **Minecraft Horizon (Rust)** on Android and Desktop combining the features of:
- **VoxelMap**: Advanced corner & big minimap with cardinal indicators, dynamic real-time cave/layer cutting, entity radar (players, hostiles, animals), in-world waypoint beacons with distance tracking, and automatic death waypoints.
- **SeedcrackerX**: In-game chunk scanner detecting worldgen structures (Desert Pyramids, Jungle Temples, Swamp Huts, Buried Treasures, End Pillars, Dungeons), mathematical LCG reversal and 48-bit lifting, version adaptation (Pre-1.18 vs 1.18+ vs 1.21+), and 64-bit world seed cracking via SHA-256 hashed seeds.

## Features

1. **Minimap & Real-Time Layer Cutting (Cave Mode)**:
   - Dynamic real-time layer cutting: automatically cuts roofs at player eye height (`Layer::Below(head)`) when entering caves, underground or the Nether.
   - Dual Map Display: Normal corner map or Big WorldMap centered mode for touchscreen navigation.
   - Live XYZ coordinates, current layer (`[Cave Y: 45]`), and biome label.

2. **Entity Radar**:
   - Classifies loaded mobs into Players (Cyan), Monsters (Red), Animals (Green), and Neutral (Gold).
   - Relative elevation indicators (`↑` above, `↓` below, `=` level).

3. **Waypoints & Auto-Death**:
   - Waypoints rendered both on the minimap and projected on the HUD with distance in meters.
   - Automatic death waypoint recorded whenever player health reaches 0.
   - Persistent storage in Android device storage without mixing data between worlds.

4. **SeedcrackerX Engine in Rust**:
   - Pure Rust implementation of Java LCG reversal (`0x5DEECE66D`, `0xDFE05BCB1365`).
   - 19-bit lower bits modular reduction followed by 48-bit structure seed candidate collection.
   - Automatic 64-bit world seed lifting using the server's SHA-256 hashed seed (`SpawnInfo.seed`).
   - Background multi-threaded processing keeping 60 FPS on mobile.

5. **Minecraft Bedrock Touch UI**:
   - Quick action bar with touch buttons: `[Map]`, `[Cave]`, `[+WP]`, `[Radar]`, `[Crk]`, `[Run]`, `[Del]`.
   - Centered modal dialog for Seedcracker status, entropy progress bar, and seed reveal.

6. **Universal Android Storage**:
   - Works across all Android devices (Samsung, Xiaomi, Pixel, Motorola, etc.).
   - Stores isolated data per world in `/sdcard/Documents/HorizonVoxelCracker/worlds/<world_id>/`:
     - `info.json`
     - `waypoints.json`
     - `seedcracker.json`
