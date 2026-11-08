# 📐 HorizonMatica for Minecraft Horizon Rust (Starling Android)

> **Litematica reescrito en puro Rust (wgpu) para el port de Android Starling, adaptado a pantalla táctil al estilo Minecraft Bedrock Edition.**

---

## 🌟 Descripción General

**HorizonMatica** es la recreación completa y nativa del legendario mod **Litematica** (creado originalmente por *masa* en Java) diseñado específicamente para **Minecraft Horizon Rust / Starling Android**.

Al igual que en Java, permite cargar esquemas de construcción tridimensionales, rotarlos, espejarlos, proyectarlos en el mundo con cajas de selección y resaltado de bloques 3D (`NativeBlockRim`), verificar en tiempo real los bloques correctos y faltantes, y calcular la lista de materiales comparada con el inventario del jugador.

Toda la interfaz y los controles han sido adaptados para **pantallas táctiles de dispositivos móviles Android** siguiendo el diseño ergonómico de **Minecraft Bedrock Edition**.

---

## 📱 Experiencia Táctil Bedrock (Touch Controls)

### 1. Botón Flotante Superior (Pill Toggle)
- Ubicado en la parte superior central de la pantalla: **`[▼ HZ-MAT]`** / **`[▲ HZ-MAT]`**.
- Un toque expande o contrae la **Barra de Acceso Rápido Bedrock**.
- No estorba los controles táctiles de movimiento ni los menús del juego.

### 2. Barra de Acceso Rápido Bedrock (Quick Action Bar)
Al tocar el botón superior, se despliega una barra horizontal con botones estilo Bedrock:
- **`[📁 Load]`**: Abre el Explorador de Archivos modal para elegir un esquemático.
- **`[📐 Layer: ALL/Y=n/Range]`**: Alterna entre ver toda la estructura, una sola capa horizontal ($Y$), o un rango de capas.
- **`[▲]` / `[▼]`**: Sube o baja la capa visible actual de forma inmediata.
- **`[🔄 Rot: 0°/90°/180°/270°]`**: Rota la orientación del esquemático 90° en el sentido de las agujas del reloj.
- **`[🪞 Mir: NONE/X/Z]`**: Espeja la estructura sobre el eje X (Front/Back) o el eje Z (Left/Right).
- **`[🔍 Ver: ON/OFF]`**: Activa/Desactiva el verificador en tiempo real.
- **`[📦 Mat]`**: Abre la ventana modal con la Lista de Materiales y conteo de inventario.
- **`[📍 Pos]`**: Traslada instantáneamente el punto de origen del esquemático a la posición de tus pies.
- **`[❌ Clr]`**: Descarga el esquemático activo y limpia los contornos visuales.

### 3. Chip Informativo del Verificador (HUD Chip)
Cuando un esquemático está activo y el verificador encendido, aparece un chip translúcido superior:
```text
[HZ-MAT] Starter Cottage (Y=All)
✓ 45  ✗ 9  ⚠ 0  + 0 (83%)
```
- **✓ (Verde)**: Bloques colocados correctamente.
- **✗ (Cian)**: Bloques faltantes por colocar.
- **⚠ (Rojo/Naranja)**: Bloques incorrectos o con estado/orientación errónea.
- **+ (Magenta)**: Bloques extra que no pertenecen a la estructura.

### 4. Diálogos Modales Táctiles (Bedrock Modals)
- **Explorador de Esquemáticos**:
  - Escanea `/sdcard/Documents/Horizonmatica/schematics/`.
  - Muestra nombres de archivo, tamaños y botones táctiles **`[Load]`**.
  - Botones de desplazamiento **`[▲]`** / **`[▼]`** y botón **`[Rescan]`**.
- **Lista de Materiales**:
  - Tabla organizada con columnas: **Ítem | Necesarios | En Inventario | Faltantes**.
  - Codificación por color: **Verde** si ya tienes suficientes ítems, **Amarillo/Rojo** si te faltan.
  - Resumen total al pie de la ventana.

---

## 🎨 Contornos Visuales 3D en el Mundo (`NativeBlockRim`)

HorizonMatica aprovecha el pipeline de renderizado nativo `NativeBlockRim` de Horizon Rust para proyectar contornos con profundidad (*depth-tested*) sobre los bloques del mundo:

| Tipo de Desajuste | Color ARGB | Significado |
|---|---|---|
| **Missing (Faltante)** | `0xFF33B3E6` (Cian) | El esquemático requiere un bloque aquí, pero hay aire. |
| **Wrong Block (Bloque Erróneo)** | `0xFFFF3333` (Rojo Brillante) | Hay un bloque colocado pero es del tipo equivocado. |
| **Wrong State (Orientación Errónea)** | `0xFFFF9010` (Naranja) | Es el bloque correcto pero mira a otra dirección. |
| **Extra (Bloque Sobrante)** | `0xFFFF4CE6` (Magenta) | Hay un bloque dentro del área que no pertenece al diseño. |
| **Bounding Box** | `0xFFFFD700` (Dorado) | Esquinas de la caja delimitadora del esquemático activo. |

---

## 📂 Formatos de Esquemáticos Compatibles

HorizonMatica soporta de forma universal:
1. **`.litematic`** (Litematica v5, v6 y v7):
   - Soporte multirregión (`Regions`).
   - Empaquetado de bits continuo que cruza límites de 64 bits (`LitematicaBitArray`).
   - Paleta dinámica con todas las propiedades de estado de bloques.
2. **`.schem`** (Sponge Schematics):
   - Lectura de paletas mapeadas y streams de bytes VarInt.
3. **`.nbt`** (Estructuras Vanilla de Mojang):
   - Lectura de bloques y paletas de archivos generados por bloques de estructuras.

### Ubicación de Archivos en Android:
```text
/sdcard/Documents/Horizonmatica/schematics/
```
*(Si la carpeta está vacía al iniciar el juego, HorizonMatica crea automáticamente un esquemático de prueba llamado `starter_cottage.litematic` para que puedas probar todas las funciones de inmediato).*

---

## 💬 Comandos en el Chat

Puedes usar `/horizonmatica`, `/hzm` o `/litematica`:

- `/hzm load <nombre_archivo>`: Carga un esquemático y lo coloca en tus coordenadas actuales.
- `/hzm unload`: Descarga el esquemático activo.
- `/hzm list`: Lista los archivos disponibles en la carpeta de esquemáticos.
- `/hzm layer <all|single|range> [y_min] [y_max]`: Configura el corte de capas visible.
- `/hzm rotate [0|90|180|270]`: Rota la orientación del esquemático.
- `/hzm mirror [none|x|z]`: Espeja la estructura sobre el eje X o Z.
- `/hzm pos [here|x y z]`: Mueve el origen del esquemático a tu posición o a coordenadas fijas.
- `/hzm verify [on|off]`: Activa o apaga la verificación en tiempo real.
- `/hzm materials`: Imprime en el chat el resumen de materiales requeridos.
- `/hzm help`: Muestra el menú de ayuda.

---

## 🏗️ Arquitectura de Módulos (Rust)

```text
crates/horizon-client/src/game/android_bundled/horizonmatica/
├── container.rs     # LitematicaBitArray (64-bit spanning bitpacking) y contenedor 3D
├── format.rs        # Decodificador NBT universal (.litematic, .schem, .nbt) con flate2 GZIP
├── placement.rs     # Motor de rotación (0/90/180/270), espejo (X/Z) y corte de capas
├── verifier.rs      # Verificador espacial en tiempo real y clasificación de bloques
├── materials.rs     # Agregador de lista de materiales y consulta de inventario del jugador
├── storage.rs       # Gestor de almacenamiento en /sdcard/Documents/Horizonmatica/schematics
├── touch_ui.rs      # Lógica táctil Bedrock, hitboxes y enrutador de toques
├── hud.rs           # Renderizado del botón flotante, barra rápida Bedrock y modales
├── visuals.rs       # Proyección de NativeBlockRim (Cian, Rojo, Naranja, Magenta, Dorado)
├── commands.rs      # Intérprete de comandos /hzm y /litematica
├── runtime.rs       # Singleton global HorizonMaticaRuntime y sincronización de hilos
└── adapters/        # Adaptadores estáticos para el sistema de hooks de Starling/Horizon
    ├── hzm_hud.rs       # Enlace con native_hud
    ├── hzm_key.rs       # Enlace con native_key y toques de pantalla táctil
    ├── hzm_tick.rs      # Enlace con native_tick para verificación throttled
    ├── hzm_visuals.rs   # Enlace con native_visuals y NativeBlockRim
    ├── hzm_command.rs   # Enlace con native_command
    ├── hzm_list.rs      # Enlace con Options > Mods list
    ├── hzm_settings.rs  # Etiquetas de configuración en el menú
    └── hzm_click.rs     # Clics de configuración en el menú
```

---

## 📜 Licencia & Atribución

- **HorizonMatica** es software libre licenciado bajo **LGPL-3.0**.
- Basado en los conceptos y estructuras de datos de **Litematica** por *masa* (Masa's Mods).
- Totalmente integrado en el árbol de mods empaquetados estáticos de **Starling Android** (`android_bundled`).
