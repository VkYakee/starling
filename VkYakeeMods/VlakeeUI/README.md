# VlakeeUI: Atajos Rápidos de Bedrock, SmartF3 More (Speedrun) y Crafteo Rápido para Starling

> **Reescrito en Rust por VkYakee para el motor Minecraft Starling (Horizon Android Rust).**  
> Inspirado en el aclamado pack de Bedrock **Déesse UI** por `@Itzriyo157`.

---

## 🏛️ Filosofía de Diseño: Pantalla Principal Java + Opciones Bedrock en Juego

Uno de los requerimientos fundamentales en el diseño de **VlakeeUI** es la separación nítida entre la interfaz del menú de inicio y la experiencia dentro del juego:

1. **Pantalla Principal 100% Java Edition (Conservada)**:
   * La pantalla de título / inicio de Minecraft en Starling conserva el diseño clásico y limpio de **Minecraft Java Edition** (panorama 3D original de Java, botones de Un Jugador / Multijugador / Opciones, tipografía vanilla y logo Java).
   * **No** se reemplaza la pantalla principal con la interfaz de Bedrock (sin Dressing Room, ni marketplace, ni paneles intrusivos en el menú inicial).

2. **Opciones y Atajos Únicamente Dentro del Juego (In-Game Only)**:
   * Todas las comodidades y atajos que hicieron famoso a Déesse UI se activan **estrictamente una vez cargado el mundo**:
     - Botón de More: panel avanzado de F-keys (`F1`, `F3+G`, `F3+B`, `F3+H`, `F3+N`, `F3+A`, `F3+C`, `Shift+F3`).
     - Botón rápido de apagado **F3 Maestro**: apaga tanto SmartF3 como el F3 nativo de Starling con 1 toque.
     - Atajos de Crafteo Rápido y Mass Crafting (crafteo de 1 toque, modo 1-Stack y recetas de speedrun).
     - Menú de pausa con opciones rápidas (Quick Settings Drawer).
     - Cambio instantáneo de perspectiva de cámara (F5) con un solo toque.
     - Selector rápido de Modo de Juego (Gamemode).
     - Ajustes rápidos de FOV (Zoom 30°, Normal 75.6°, Quake Pro 110°).
     - Atajos de Contenedores y Cofres (Move All, Drop All, Restock).
     - Bloc de Notas Temporal en juego (`Temporary Notes`).
     - Anotador de Coordenadas y Waypoints (`Coord Notes`).
     - Compartir coordenadas directamente en el chat.
     - Telemetría SmartF3, InvHUD, Visor de Durabilidad, Chunk Viewer 3D y Calculadora en juego.

---

## ⛔ Botón Rápido de Apagado: F3 Maestro (Master Killswitch)
Para mantener la pantalla 100% limpia sin distracciones o en grabaciones:
* **Botón `[ ⛔ F3 Maestro: ACTIVO / APAGADO ]`**:
  - Al pulsar el botón o ejecutar `/vui f3mute` / `/vk f3off`, **apaga al mismo tiempo** tanto el SmartF3 de VlakeeUI como el F3 por defecto / nativo de Starling.
  - Al volverlo a presionar, restaura instantáneamente la telemetría.
  - Comandos: `/vui f3mute`, `/vui silence`, `/vk f3off`, `/vk f3mute`.

---

## ➕ SmartF3 More: Atajos de Speedrunners y Jugadores Técnicos (F-Keys)

Diseñado especialmente para speedrunners y jugadores técnicos que requieren las combinaciones de depuración de Minecraft Java en pantallas táctiles de Android:

| Tecla / Atajo | Función Técnica / Speedrun | Comando Equivalente |
| :--- | :--- | :--- |
| **`F1`** | **Modo Cinemático**: Oculta por completo el HUD, mano, crosshair y menús. | `/f1` o `/vui f1` |
| **`F3 + G`** | **Bordes de Chunks**: Muestra la rejilla 3D de subchunks y límites de chunk. | `/vui f3 g` o `/vui f3g` |
| **`F3 + B`** | **Hitboxes de Entidades**: Cajas de colisión de mobs (esencial para rastrear blazes, bastiones y piglins). | `/vui f3 b` o `/vui f3b` |
| **`F3 + H`** | **Tooltips Avanzados**: Muestra durabilidad numérica (`1561/1561`), IDs de objetos y tags NBT. | `/vui f3 h` o `/vui f3h` |
| **`F3 + N`** | **Espectador Rápido**: Alterna en 1 toque entre Modo Espectador y el modo anterior (Supervivencia/Creativo). | `/vui f3 n` o `/vui f3n` |
| **`F3 + A`** | **Recargar Chunks**: Fuerza el refresco y re-renderizado de todos los chunks visibles. | `/vui f3 a` o `/vui f3a` |
| **`F3 + C`** | **Copiar Coordenadas TP**: Genera el comando `/execute in <dim> run tp @s X Y Z Yaw Pitch` en el portapapeles. | `/vui f3 c` o `/vui f3c` |
| **`F3 + D`** | **Limpiar Chat**: Vacía el historial de mensajes del chat para no obstruir la visión. | `/vui f3 d` o `/vui f3d` |
| **`F3 + T`** | **Recargar Texturas**: Recarga los paquetes de recursos y texturas sin reiniciar el cliente. | `/vui f3 t` o `/vui f3t` |
| **`Shift + F3`** | **Pie Chart / Profiler**: Gráfico circular de telemetría de rendimiento y recuento de entidades. | `/vui pie` o `/vui profiler` |

*Acceso*: Botón **`➕ [ SMARTF3 MORE (Speedrun & F-Keys) ]`** en la Página 1 del menú táctil o ejecutando `/vui more`.

---

## ⚡ Atajos de Crafteo Rápido (Quick Crafting & Mass Crafting)

Inspirado en el sistema `mass_craft.json` de Déesse UI, optimiza drásticamente los tiempos de crafteo en partidas de supervivencia y speedruns en móviles:

### 1. Modos de Crafteo
* **Normal**: Craftea 1 unidad por toque.
* **Mass Craft (Crafteo Masivo)**: Craftea **todo lo que permitan los materiales del inventario** en un solo toque (equivalente a Shift+Click).
* **1 Stack (Máx 64)**: Craftea exactamente hasta completar 1 stack de 64 unidades sin sobrepasarse.
* *Alternar*: Botón `[ ⚡ Modo Crafteo ]` o comando `/vui craft mode`.

### 2. Recetas Rápidas Instantáneas
* `🪵 Madera ➔ Tablones`: Convierte troncos a tablones (1 tronco = 4 tablones) (`/vui craft planks`).
* `🥢 Tablones ➔ Palos`: Convierte tablones a palos (2 tablones = 4 palos) (`/vui craft sticks`).
* `🪑 Tablones ➔ Mesa de Crafteo`: Craftea mesas de trabajo (4 tablones = 1 mesa) (`/vui craft table`).
* `🕯️ Palos + Carbón ➔ Antorchas`: Craftea antorchas en lotes de 4 (`/vui craft torches`).
* `🧱 Lingotes ➔ Bloque Compacto`: Compacta hierro, oro, diamantes o redstone (9 lingotes = 1 bloque) (`/vui craft blocks`).
* `🪙 Bloques ➔ Lingotes`: Descompacta bloques a lingotes (1 bloque = 9 lingotes) (`/vui craft ingots`).
* `🚀 Pólvora + Papel ➔ Cohetes`: Craftea cohetes de fuegos artificiales para elytra (`/vui craft rockets`).
* `🍞 Trigo ➔ Pan`: Craftea pan para alimentación (3 trigo = 1 pan) (`/vui craft bread`).
* `🛡️ Hierro + Madera ➔ Escudo`: Craftea escudos instantáneos para combate (`/vui craft shield`).
* `🪣 Hierro ➔ Cubo`: Craftea cubos de agua/lava (3 lingotes = 1 cubo) (`/vui craft bucket`).

*Acceso*: Botón **`⚡ [ ATAJOS DE CRAFTEO RÁPIDO ]`** en la Página 3 del menú táctil o ejecutando `/vui craft`.

---

## 🎮 Opciones Dentro del Juego (In-Game Quick Options)

1. **Cambio Rápido de Cámara (F5 / 1-Tap)**: Alterna entre 1ª Persona, 3ª Persona Espalda y 3ª Persona Frontal (`/vui cam`).
2. **Selector Rápido de Gamemode**: Alterna entre Supervivencia, Creativo, Espectador y Aventura (`/vui gm`).
3. **Presets Rápidos de FOV**: Zoom 30.0° (catalejo rápido), Normal 75.6° y Quake Pro 110.0° (`/vui fov`).
4. **Distancia de Render Rápida**: 8, 12, 16 y 24 Chunks (`/vui render`).
5. **Atajos de Tiempo y Clima**: `/vui time` y `/vui weather`.
6. **Bloc de Notas Temporal en Juego (`Temporary Notes`)**: Anota tareas y coordenadas temporales (`/vui note <texto>`, `/vui notes`, `/vui clearnotes`).
7. **Anotador de Coordenadas y Waypoints (`Coord Notes`)**: Marca puntos de interés con nombre propio (`/vui mark <nombre>`, `/vui waypoints`, `/vui delmark <id>`).
8. **Compartir Coordenadas en Chat**: `/vui sharepos` o `/vui pastecoords`.
9. **Atajos de Contenedor (Quick Container)**: Mover todo a cofre, tirar todo al suelo o reabastecer hotbar (`/vui container [move|drop|restock]`).

---

## 📑 Menú Táctil de 5 Páginas en el HUD

El menú táctil de ajustes (`settings_buttons()`) cuenta con 5 páginas especializadas:
* **Página 1 (Telemetría & HUD)**:
  - `[ 📊 SmartF3: Off / Compacto / Completo ]`
  - `[ 🎒 InvHUD: Activado / Desactivado ]`
  - `[ ⚔️ Durabilidad: Visible / Oculto ]`
  - `[ 💡 Visión Nocturna: Brillo % ]`
  - `[ ⛔ F3 Maestro: ACTIVO / APAGADO ]` *(Apaga SmartF3 + F3 nativo de Starling)*
  - `[ ➕ SMARTF3 MORE (Speedrun & F-Keys) ]`
  - `[ ➡️ Ir a Página 2 ]`
* **Página 2 (Mundo & Herramientas)**:
  - `[ 🗺️ Chunk Viewer: Modo ]`
  - `[ 🔴 Convertidor Portal Nether ]`
  - `[ 🧭 Brújula de Muerte / Recuperación ]`
  - `[ 🔢 Coordenadas: Visibles / Ocultas ]`
  - `[ 💬 Chat en Pantalla: Visible / Oculto ]`
  - `[ 🧮 Calcular Stacks (64x4) ]`
  - `[ ➡️ Ir a Página 3 ]`
* **Página 3 (🎮 Opciones Dentro del Juego)**:
  - `[ 📷 Cámara (F5): 1ª / 3ª Espalda / 3ª Frontal ]`
  - `[ 🎮 Modo de Juego: Supervivencia / Creativo / Espectador / Aventura ]`
  - `[ 👁️ Campo Visión (FOV): 30° / 75.6° / 110° ]`
  - `[ 🔭 Render: 8 / 12 / 16 / 24 Chunks ]`
  - `[ 📦 Atajos Contenedor: Mover / Tirar / Restock ]`
  - `[ ⚡ ATAJOS DE CRAFTEO RÁPIDO ]`
  - `[ 📍 Guardar Coords Aquí (Waypoint) ]`
  - `[ 💬 Compartir Coordenadas en Chat ]`
  - `[ ➡️ Ir a Página 4 ]`
* **Página 4 (➕ SmartF3 More & Speedrun)**:
  - `[ 🎬 F1 (Ocultar HUD): Visible / Oculto ]`
  - `[ 🗺️ F3+G (Bordes Chunks): ON / OFF ]`
  - `[ 📦 F3+B (Hitboxes): ON / OFF ]`
  - `[ ℹ️ F3+H (Tooltips Avanzados): ON / OFF ]`
  - `[ 👻 F3+N: Alternar Modo Espectador ]`
  - `[ 🔄 F3+A: Recargar Todos los Chunks ]`
  - `[ 📋 F3+C: Copiar Coords TP ]`
  - `[ ⏱️ Shift+F3 (Pie Chart): ON / OFF ]`
  - `[ ➡️ Ir a Página 5 ]`
* **Página 5 (⚡ Crafteo Rápido / Mass Craft)**:
  - `[ ⚡ Modo Crafteo: Normal / Mass Craft / 1 Stack ]`
  - `[ 🪵 Madera ➔ Tablones ]`
  - `[ 🥢 Tablones ➔ Palos ]`
  - `[ 🪑 Tablones ➔ Mesa Crafteo ]`
  - `[ 🕯️ Palos+Carbón ➔ Antorchas ]`
  - `[ 🧱 Lingotes ➔ Bloque Compacto ]`
  - `[ 🪙 Bloques ➔ Lingotes ]`
  - `[ 🚀 Pólvora+Papel ➔ Cohetes ]`
  - `[ ⬅️ Volver a Página 1 ]`

---

## ⚖️ Créditos
* **Reescrito en Rust para Starling por**: `VkYakee`
* **Inspiración y Creador Original de Déesse UI**: `@Itzriyo157`
* **Colaboradores de Déesse UI**: `@T1M0THY`, `@Dimzcraft`, `@tlgm2308`, `The Andromedarius`, `drav0011`.
