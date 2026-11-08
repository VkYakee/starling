# Rustmatica

**Rustmatica** es una reimplementación completa, de alto rendimiento y puramente en **Rust** de [Litematica](https://github.com/maruohon/litematica).

Proporciona capacidades avanzadas de manejo de esquemas de Minecraft (`.litematic`), conteo y optimización de listas de materiales, verificador de bloques en tiempo real, transformaciones de colocación (rotación y espejado), modos de herramientas, protocolos *Easy Place*, y un motor de gestión de recursos y traducción que aprovecha directamente los assets originales de Litematica.

---

## 🚀 Características Principales

1. **Manipulación de Esquemas `.litematic`**:
   - Decodificación y codificación completa de NBT comprimido con GZIP (`LitematicaSchematic`).
   - Implementación exacta de `LitematicaBitArray` (empaquetado a nivel de bits de profundidad variable de 1 a 32 bits a través de arrays `long`).
   - Soporte para sub-regiones múltiples con posiciones y tamaños arbitrarios.
   - Preservación de Tile Entities, Entities, y Pending Block/Fluid Ticks.
   - Conversión de estructuras nativas de Minecraft (`.nbt`) a `.litematic`.

2. **Verificador de Esquemas (`SchematicVerifier`)**:
   - Comparación precisa de bloques entre el esquema colocado y el mundo real.
   - Clasificación granular de discrepancias:
     - `Missing`: Falta el bloque en el mundo.
     - `Extra`: Bloque sobrante en el mundo.
     - `WrongBlock`: Tipo de bloque incorrecto.
     - `WrongState`: Mismo bloque pero propiedades diferentes (ej. orientación, escaleras invertidas).
     - `DiffBlock`: Variación permitida o equivalente.
     - `CorrectState`: Coincidencia exacta.
   - Filtrado por capas (`LayerRange`).

3. **Lista de Materiales y Analizador de Inventario (`MaterialList`)**:
   - Agregación y conteo exacto de bloques y entidades.
   - Conversión de estados a ítems requeridos (manejo de antorchas en pared, carteles, redstone, cubos).
   - Cálculo automático en stacks (64, 16, 1), cajas de shulker y cofres dobles.
   - Rastreo de progreso de inventario (`InventoryTracker`): disponibles vs faltantes.
   - Exportación a tablas formateadas, JSON y CSV.

4. **Sistema de Colocación y Transformaciones (`SchematicPlacement`)**:
   - Transformaciones espaciales completas: Rotación (0°, 90°, 180°, 270°) y Espejado (Frente-Atrás, Izquierda-Derecha).
   - Bloqueo de coordenadas y ajuste fino (*nudging*).
   - Administrador de múltiples colocaciones simultáneas (`SchematicPlacementManager`).

5. **Herramientas y Modos de Operación (`ToolMode`)**:
   - `AreaSelection`: Selección de regiones tridimensionales por cajas (`BoxRegion`, `AreaSelection`).
   - `SchematicPlacement`: Posicionamiento del esquema.
   - `Fill`, `ReplaceBlock`, `PasteSchematic`, `GridPaste`, `Move`, `Delete`, `Rebuild`.
   - Soporte de restricciones y atajos de teclado (*Hotkeys*).

6. **Algoritmo *Easy Place***:
   - Detección de coincidencia de ítems sostenidos en mano.
   - Cálculo de compensación de clics (*click offset*) para codificar orientación (facing, half, axis) compatible con protocolos V2 y V3.

7. **Uso de Assets y Multi-Idioma (`AssetManager`)**:
   - Carga y utiliza directamente las carpetas copiadas de Litematica:
     - `assets/litematica/lang/`: Traducciones en 15 idiomas (`en_us`, `es_es`, `ja_jp`, `zh_cn`, `ru_ru`, etc.).
     - `assets/litematica/blockstates/`: Definiciones de estados de bloques.
     - `assets/litematica/models/` y `textures/`: Modelos y texturas GUI.
     - `assets/litematica/icon.png`: Icono oficial.

8. **Protocolo de Red Servux (`ServuxLitematicaPacket`)**:
   - Protocolo de paquetes binarios para sincronización cliente/servidor.

---

## 📦 Estructura del Proyecto

```text
rustmatica/
├── Cargo.toml
├── assets/                          # Assets copiados de Litematica
│   └── litematica/
│       ├── blockstates/
│       ├── icon.png
│       ├── lang/                    # 15 idiomas (en_us, es_es, etc.)
│       ├── models/
│       ├── shaders/
│       └── textures/
├── LICENSE.txt
├── README.md
└── src/
    ├── lib.rs                       # Exportaciones de la librería
    ├── main.rs                      # Interfaz de línea de comandos (CLI)
    ├── assets.rs                    # Gestor de assets y traducción multilingüe
    ├── config/                      # Subsistema de configuración
    │   ├── generic.rs
    │   ├── visuals.rs
    │   ├── colors.rs
    │   ├── hotkeys.rs
    │   ├── info_overlays.rs
    │   └── mod.rs
    ├── schematic/                   # Núcleo de esquemas
    │   ├── bit_array.rs             # LitematicaBitArray bit-packing
    │   ├── palette.rs               # BlockState y BlockStatePalette
    │   ├── container.rs             # LitematicaBlockStateContainer
    │   ├── metadata.rs              # SchematicMetadata
    │   ├── litematic.rs             # Lector y escritor de archivos .litematic
    │   ├── placement.rs             # SchematicPlacement y transformaciones
    │   ├── verifier.rs              # SchematicVerifier (analizador de discrepancias)
    │   ├── projects.rs              # Proyectos de esquemas y versiones (VCS)
    │   ├── conversion.rs            # Convertidores de estructuras
    │   └── mod.rs
    ├── selection/                   # Gestión de áreas y cajas
    │   ├── box_region.rs            # BoxRegion 3D
    │   ├── area_selection.rs        # AreaSelection multi-caja
    │   ├── manager.rs               # SelectionManager
    │   └── mod.rs
    ├── materials/                   # Lista de materiales y exportadores
    │   ├── material_list.rs         # Cálculo de stacks, shulkers y cofres
    │   ├── inventory.rs             # Seguimiento de inventario y faltantes
    │   ├── exporter.rs              # Exportación a Tabla, JSON y CSV
    │   └── mod.rs
    ├── tool/                        # Modos de herramienta
    │   ├── mode.rs                  # Enum ToolMode y lógica
    │   └── mod.rs
    ├── util/                        # Utilidades matemáticas y espaciales
    │   ├── pos.rs                   # BlockPos, Vec3i, Rotation, Mirror
    │   ├── layer.rs                 # Modos y rangos de capas (LayerRange)
    │   ├── easy_place.rs            # Orientaciones y protocolos Easy Place
    │   └── mod.rs
    └── network/                     # Red y protocolo Servux
        ├── packet.rs                # ServuxLitematicaPacket
        └── mod.rs
```

---

## 🛠️ Uso de la CLI

### 1. Inspeccionar un archivo `.litematic`
```bash
cargo run -- info ruta/al/esquema.litematic
```

### 2. Generar lista de materiales (en consola, JSON o CSV)
```bash
cargo run -- materials ruta/al/esquema.litematic
cargo run -- materials ruta/al/esquema.litematic --format json --output materiales.json
cargo run -- materials ruta/al/esquema.litematic --format csv --output materiales.csv
```

### 3. Crear un nuevo esquema `.litematic`
```bash
cargo run -- create "MiConstruccion" salida.litematic -d 10 5 10
```

### 4. Verificar un esquema
```bash
cargo run -- verify salida.litematic
```

### 5. Inspeccionar y consultar assets copiados (I18N / Recursos)
```bash
# Listar idiomas y recursos encontrados
cargo run -- assets

# Ver traducciones en español u otros idiomas
cargo run -- lang --locale es_es
cargo run -- lang --locale es_es litematica.gui.label.schematic_verifier.verifier
```

### 6. Volcar la configuración
```bash
cargo run -- config --dump
```

---

## 🧪 Pruebas Automatizadas

Para ejecutar la suite de pruebas unitarias:
```bash
cargo test
```
Resultados:
- `test_bit_array_roundtrip` (Prueba de compresión y empaquetado bit a bit de 1 a 16 bits)
- `test_container_get_set` (Prueba de contenedor de bloques y coordenadas 3D)
- `test_schematic_save_and_load` (Prueba de ida y vuelta de lectura/escritura en disco con NBT y GZIP)
- `test_verifier_accuracy` (Prueba del analizador de estados de bloques faltantes, erróneos y extra)
- `test_transformations` (Prueba de rotaciones de 90°/180°/270° y reflejos)
- `test_assets_loader` (Prueba de carga y localización de recursos en los 15 idiomas)
