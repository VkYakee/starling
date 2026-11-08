# VkYakeeMods: Framework y Ecosistema de Mods en Rust para Starling (Horizon Android)

> **Este framework y sus componentes fueron Reescritos en Rust por VkYakee para el motor Minecraft Starling (Horizon Android Rust).**

`VkYakeeMods` es un framework de alto rendimiento y arquitectura desacoplada diseñado para portar, reescribir y ejecutar modificaciones nativas de Minecraft Java y Bedrock Edition sobre el motor **Starling (Horizon)** en plataformas móviles (Android / Termux) y escritorios Linux.

Actualmente incluye tres mods de nivel profesional completamente integrados:
1. **SeedCracker**: Motor de ingeniería inversa de semillas de mundo ($2^{64}$ posibilidades) mediante retículos LLL, análisis de Bedrock y descarte predictivo.
2. **HorizonMatica**: Reescritura nativa en Rust de Litematica para Starling con **100% de paridad funcional** con el Litematica original de Java y Rustmatica:
   - Soporte multiformato nativo (`.litematic`, `.schem`, `.schematic`, `.nbt`).
   - Flujo interactivo en 3 pasos con botón `[ ▶ INICIAR HOLOGRAMA ]`.
   - Mini-HUD flotante de 1 botón `[ 🛠️ Herramientas ]` que evita obstruir los controles táctiles de Android.
   - Panel desplegable de herramientas: Rotación, Espejo, Bloqueo de colocación, Desplazamiento Y/X/Z, Opacidad dinámica.
   - **Verificador de Bloques (`SchematicVerifier`)**: Detecta bloques faltantes, bloques incorrectos, estados erróneos y bloques sobrantes comparando el mundo con la esquemática.
   - **Conteo y Rastreo de Inventario (`InventoryTracker`)**: Compara en tiempo real los materiales del jugador con los requeridos, indicando faltantes.
   - **Exportador de Materiales (`MaterialListExporter`)**: Exporta tablas a CSV, JSON y texto plano directamente a `/sdcard/VkYakeeMods/HorizonMatica/exports/`.
   - **Corte de Capas Multieje**: Slicing en los tres ejes cartesianos (Eje Y vertical, Eje X este-oeste, Eje Z norte-sur) con modos Todas, Capa Individual y Rango.
   - **Protocolo EasyPlace (V2/V3/Slab)**: Cálculo de offsets de clic para orientación automática de escaleras, troncos, losas y observadores.
   - **Proyectos y Versionado**: Historial de versiones y commits instantáneos de esquemáticas.
   - **Herramientas de Creación**: Pos1, Pos2, caja magenta de selección 3D y guardado directo a `.litematic`.
3. **VlakeeUI**: Reescritura en Rust nativo del aclamado pack de utilidades de Bedrock **Déesse UI** (`deesse_ui`):
   - **Preservación de la Pantalla Principal Java Edition**: La pantalla de inicio/título de Minecraft mantiene 100% la estética clásica, minimalista y limpia de Java Edition (panorama vanilla y menús Java). Las mejoras y atajos de Bedrock operan **únicamente dentro del juego**.
   - **Botón Rápido de Apagado F3 Maestro**: Un solo botón para apagar tanto SmartF3 como el F3 nativo de Starling simultáneamente (`/vk f3off`, `/vui f3mute`).
   - **SmartF3 More (Atajos de Speedrunners y Jugadores Técnicos)**: Botón de "More" con todas las combinaciones F-Keys (`F1`, `F3+G` bordes de chunks, `F3+B` hitboxes de entidades, `F3+H` tooltips avanzados y durabilidad numérica, `F3+N` espectador rápido, `F3+A` recargar chunks, `F3+C` copiar coords TP, `Shift+F3` profiler pie chart).
   - **Atajos de Crafteo Rápido & Mass Crafting**: Modos Normal, Mass Craft (craftea todo el inventario con 1 toque) y 1-Stack (máx 64 unidades). Recetas de un clic para madera, palos, mesas de trabajo, antorchas, lingotes/bloques compactos y cohetes.
   - **Opciones Dentro del Juego (Déesse UI In-Game Options)**: Menú de pausa interactivo con cajón de ajustes rápidos, cambio de perspectiva de cámara (F5) con un toque, selector rápido de Gamemode (Supervivencia, Creativo, Espectador, Aventura), presets de FOV (Zoom 30°, Normal 75.6°, Quake Pro 110°) y selector de distancia de renderizado.
   - **Bloc de Notas Temporal & Waypoints**: Bloc de notas en juego (`/vui note`), marcador de coordenadas (`/vui mark`) y botón para compartir coordenadas formateadas en el chat (`/vui sharepos`).
   - **Atajos de Contenedor (Quick Container)**: Mover todo al cofre, tirar todo al suelo (drop all) o reabastecer la hotbar (restock).
   - **SmartF3 / Debug Screen**: Telemetría compacta y completa en pantalla sin recurrir a menús pesados de depuración. Coordenadas precisas, orientación con ángulos yaw/pitch, bioma, nivel de luz, ciclo lunar y contador de días.
   - **Convertidor de Coordenadas de Portales del Nether**: Cálculo instantáneo y en tiempo real de coordenadas exactas para portales sincronizados (`Overworld -> Nether: X/8, Z/8` y `Nether -> Overworld: X*8, Z*8`).
   - **InvHUD & Alerta de Inventario Lleno**: Monitorea slots ocupados (Hotbar 9 + Mochila 27 = 36 slots), lanza alertas visuales de advertencia (`§c¡Tu inventario está lleno!`) y muestra información del ítem en mano.
   - **Visor de Durabilidad**: Monitoreo de herramientas en mano principal, offhand y las 4 piezas de armadura con avisos de durabilidad crítica (< 15%).
   - **Chunk Viewer & Delimitador 3D**: Detección de centro de chunk, bordes (0/15), esquinas y renderizado de cajas de colisión alámbricas 3D de subchunks.
   - **Atajos y Toggles Rápidos**: Visión nocturna con ajuste de brillo (25% a 100%), brújula de recuperación (coordenadas y distancia a la última muerte), reloj/brújula integrados y ocultación de chat/scoreboard.
   - **Calculadora en Juego**: Evaluación aritmética directa (`/vui calc <expr>`), conversión de bloques a stacks, shulkers y cofres dobles.
4. **VkMap**: El cuarto mod del ecosistema Starling, combinando lo mejor de los mapas de Java (`Dynmap`, `Squaremap`, `VoxelMap`, `XaeroPlus`, `BlueMap`, `Conflux-Map`):
   - **3 Tamaños de Minimapa Táctil**: Pequeño (96px), Mediano nativo (144px) y Pantalla Completa táctil (280px).
   - **Widgets Flotantes con Formas de Lupa**: `[ 🔍+ Ampliar ]`, `[ 🔍- Reducir ]`, `[ 🔎 Zoom: 1.0x ]` y `[ 🗺️ Menú ]`.
   - **Modos de Visibilidad de Radar**: `LineOfSight` (cono de visión frontal), `AllLayers` (rayos X omnisciente), `SurfaceOnly` (misma superficie Y±5) y `Fair-Play` (anti-cheat para torneos).
   - **Analizador Subterráneo de Cuevas ($Y = -64..0$)**: Localiza Trial Chambers y Ancient Cities en capas negativas.
   - **Sinergia Bidireccional con SeedCracker**: Comparte estructuras descubiertas para el cracking y proyecta estructuras predichas (`[PRED]`) que se verifican (`[VERIF]`) al cargar bloques.
   - **Almacenamiento Local Aislado en Android**: Guarda cada mundo/servidor por separado en `/sdcard/VkYakeeMods/VkMap/worlds/{world_id}/` y exporta imágenes completas a PNG y datasets a JSON en `/sdcard/VkYakeeMods/VkMap/exports/`.

---

## 🧭 Índice
1. [Visión General](#-visión-general)
2. [VkMap: Motor de Mapa, Radar y Detección Subterránea](#-vkmap-motor-de-mapa-radar-y-detección-subterránea)
3. [Matriz de Paridad: HorizonMatica vs Litematica Original (Java)](#-matriz-de-paridad-horizonmatica-vs-litematica-original-java)
4. [VlakeeUI: Atajos de Bedrock, SmartF3 y Herramientas](#-vlakeeui-atajos-de-bedrock-smartf3-y-herramientas)
   - [SmartF3 y Pantalla de Telemetría](#smartf3-y-pantalla-de-telemetría)
   - [Convertidor de Portales del Nether](#convertidor-de-portales-del-nether)
   - [InvHUD y Alerta de Inventario Lleno](#invhud-y-alerta-de-inventario-lleno)
   - [Visor de Durabilidad de Armadura y Herramientas](#visor-de-durabilidad-de-armadura-y-herramientas)
   - [Chunk Viewer y Rejilla Alámbrica 3D](#chunk-viewer-y-rejilla-alámbrica-3d)
   - [Calculadora en Juego y Conversor de Stacks](#calculadora-en-juego-y-conversor-de-stacks)
   - [Visión Nocturna y Brújula de Recuperación](#visión-nocturna-y-brújula-de-recuperación)
4. [Estructura del Almacenamiento Local en Android](#-estructura-del-almacenamiento-local-en-android)
5. [HorizonMatica: Flujo Interactivo y Herramientas](#-horizonmatica-flujo-interactivo-y-herramientas)
   - [¿Qué botón uso para iniciar el holograma? (Flujo en 3 Pasos)](#qué-botón-uso-para-iniciar-el-holograma-flujo-en-3-pasos)
   - [Mini-HUD Flotante de 1 Botón: [🛠️ Herramientas]](#mini-hud-flotante-de-1-botón-️-herramientas)
   - [Panel de Herramientas y Edición](#panel-de-herramientas-y-edición)
   - [Auditoría con el Verificador de Esquemáticas (SchematicVerifier)](#auditoría-con-el-verificador-de-esquemáticas-schematicverifier)
   - [Inventario en Tiempo Real y Exportación CSV/JSON](#inventario-en-tiempo-real-y-exportación-csvjson)
   - [Corte de Capas Multieje (Ejes Y, X, Z)](#corte-de-capas-multieje-ejes-y-x-z)
   - [Creación de Nuevos Litematics (Pos1, Pos2, Guardar)](#creación-de-nuevos-litematics-pos1-pos2-guardar)
   - [Mini-Renderizador 3D Isométrico y 2D](#mini-renderizador-3d-isométrico-y-2d)
6. [SeedCracker en Rust: Guía y Algoritmos](#-seedcracker-en-rust-guía-y-algoritmos)
   - [1. Inversión LCG de 48 bits](#1-inversión-del-generador-lineal-congruencial-java-lcg-de-48-bits)
   - [2. Reducción de Retículos LLL y Babai CVP](#2-reducción-de-retículos-lll-y-el-problema-del-vector-más-cercano-babai-cvp)
   - [3. Cracker Jerárquico de Bedrock Bit-Slice](#3-cracker-jerárquico-de-bedrock-por-rebanado-de-bits-bit-slice)
   - [4. Base de Datos de Formaciones de Tesoros](#4-base-de-datos-precomputada-de-tesoros-enterrados)
   - [5. Motor de Biomas y Estructuras C Cubiomes](#5-integración-del-motor-de-biomas-y-estructuras-c-cubiomes)
   - [6. Sistema Híbrido de Descarte Predictivo (`SeedVerifier`)](#6-sistema-híbrido-de-descarte-predictivo-y-verificación-en-terreno)
7. [Ergonomía Táctil y Prevención de Solapamiento en Android](#-ergonomía-táctil-y-prevención-de-solapamiento-en-android)
8. [Seguridad y Anti-Detección en Servidores (100% Client-Side Stealth)](#-seguridad-y-anti-detección-en-servidores-100-client-side-stealth)
9. [Comandos y Accesos Rápidos](#-comandos-y-accesos-rápidos)
10. [Créditos y Licencias](#-créditos-y-licencias)

---

## 🌟 Visión General

A diferencia de los entornos Java clásicos (Fabric / Forge), donde los mods operan sobre bytecode mediante Mixins y sufren sobrecarga de recolección de basura (GC) y llamadas JNI/Panama, **VkYakeeMods** se ejecuta en código máquina puro compilado con LLVM en Rust:
* **Cero overhead de recolección de basura (No GC)**: Esencial para procesadores móviles ARM Cortex-A y Snapdragon bajo altas cargas de chunks y bloques.
* **Integración nativa in-process**: Menos de 10 líneas de código integradas en el motor Starling (`crates/horizon-client/src/game/native_hooks.rs`).
* **Sub-hilos paralelos**: Los cálculos pesados de retículos LLL, escaneo de Bedrock, auditorías de bloques y conversiones NBT se ejecutan en segundo plano vía `rayon` sin penalizar los 60/120 FPS de Starling Vulkan.

---

## 📱 VlakeeUI: Atajos de Bedrock, SmartF3 y Herramientas

`VlakeeUI` adapta a Rust nativo para Starling el paquete de interfaz de Bedrock **Déesse UI**, conservando sus assets originales (texturas, sonidos, materiales e iconos en `VkYakeeMods/VlakeeUI/assets/`) y programando la lógica en código Rust ultra eficiente.

### 🏛️ Preservación de la Pantalla Principal Java Edition
* **Pantalla de Inicio Vanilla Java**: La pantalla principal de Starling (menú de título, selector de mundos, servidores y opciones) se mantiene **100% idéntica a Minecraft Java Edition**.
* **Aislamiento en Juego (In-Game Only)**: Todas las comodidades y atajos de Bedrock operan **exclusivamente una vez dentro del mundo**, evitando sobrecargar o desnaturalizar la interfaz de inicio clásica de Java.

### 🎮 Opciones Dentro del Juego (Déesse UI In-Game Options)
* **Cambio Rápido de Cámara (F5 / 1-Toque)**: Alterna entre 1ª Persona, 3ª Persona Espalda y 3ª Persona Frontal sin entrar a los ajustes de video (`/vui cam` o tecla F5).
* **Selector Rápido de Gamemode**: Cambia en un toque entre Supervivencia, Creativo, Espectador y Aventura (`/vui gm`).
* **Presets de FOV (Campo de Visión)**: Ajusta instantáneamente tu visión a Zoom 30° (catalejo rápido), Normal 75.6° o Quake Pro 110° (`/vui fov`).
* **Distancia de Renderizado Rápida**: Alterna entre 8, 12, 16 y 24 Chunks con un clic (`/vui render`).
* **Atajos de Tiempo y Clima**: Modifica la hora (`/vui time [day|noon|night|midnight]`) o el clima (`/vui weather [clear|rain|thunder]`).
* **Bloc de Notas Temporal en Juego (`Temporary Notes`)**: Toma notas rápidas durante tus partidas (`/vui note <texto>`, `/vui notes`, `/vui clearnotes`).
* **Anotador de Coordenadas y Waypoints (`Coord Notes`)**: Guarda tus coordenadas con nombre propio (`/vui mark <nombre>`, `/vui waypoints`, `/vui delmark <id>`).
* **Compartir Coordenadas en Chat**: Formatea y prepara tus coordenadas actuales para compartirlas de inmediato (`/vui sharepos`).
* **Atajos de Contenedor (Quick Container)**: Mover todo al cofre, tirar todo al suelo (drop all) o reabastecer la hotbar (restock) (`/vui container`).
* **Menú de Pausa Interactivo (Déesse Pause Drawer)**: Despliega un panel lateral de opciones en el menú de pausa (`/vui pause` o Página 3 de botones táctiles).

### SmartF3 y Pantalla de Telemetría
Al pulsar el botón F3 en pantalla o ejecutar `/vui f3`, alterna entre tres modos:
* **Desactivado**: HUD limpio sin datos.
* **Compacto**:
  ```text
  ┌─ [ 📊 SmartF3: Compacto ] ──────────────────────┐
  │ XYZ:   128 /  64 / -256 | Sur (South) (Hacia +Z)
  │ 🔴 Nether Portal Target: [16, 64, -32]
  │ Bioma: Plains         | FPS:  60 | Día 14 (12:30)
  └──────────────────────────────────────────────┘
  ```
* **Completo (SmartF3)**:
  Muestra versión de Starling, coordenadas flotantes precisas, bloque actual, chunk local y global `[cx, cz en subchunk rx, ry, rz]`, rotación con ángulos yaw y pitch exactos, nivel de luz (Cielo y Bloque), fase lunar de 8 estados con aviso de Luna Llena (`Full Moon §a[!]`) y porcentaje de progreso de nivel de XP actual.

### Convertidor de Portales del Nether
Calcula automáticamente las coordenadas del portal en la otra dimensión:
* Si estás en el **Overworld**: `Overworld [X, Y, Z] ➔ Nether [X/8, Y, Z/8]`.
* Si estás en el **Nether**: `Nether [X, Y, Z] ➔ Overworld [X*8, Y, Z*8]`.
* Toggable con `/vui nether` o desde el menú táctil.

### InvHUD y Alerta de Inventario Lleno
* Monitorea la capacidad total de slots: **9 slots de Hotbar** + **27 slots de Mochila** = **36 slots**.
* Muestra slots ocupados vs vacíos en tiempo real:
  ```text
  🎒 Inventario: 34/36 ocupados (2 vacíos) - §e⚠️ Casi lleno
     Hotbar: 8/9 | Mochila: 26/27
  ```
* Cuando el jugador se queda sin espacio, dispara de inmediato el aviso:
  **`§c¡INVENTARIO LLENO! (0 espacios)`**
* **Información de Objeto en Mano**: Indica cuántos bloques/items de ese tipo tienes en todo el inventario y cuántos slots ocupan (ej: `Mano: Cobblestone (192 items en 3 slots)`).

### Visor de Durabilidad de Armadura y Herramientas
* Monitorea en vivo la herramienta en la mano principal (`Pickaxe: 1340/1561 (86%)`).
* Lanza alertas en amarillo/rojo si la herramienta baja del 15% de vida: **`§c[¡DURABILIDAD CRÍTICA!]`**.
* Muestra el escudo de la mano secundaria y las 4 piezas de armadura con porcentajes en una sola línea compacta: `🛡️ Armadura: [H:88% | C:93% | L:91% | B:89%]`.

### Chunk Viewer y Rejilla Alámbrica 3D
Alterna con `/vui chunk [mid|edge|corner|grid]` o desde la página 2 de botones táctiles:
* **Centro de Chunk**: Avisa y resalta el área central 2x2 del chunk `[X/Z = 7..8]` con caja verde 3D.
* **Borde de Chunk**: Avisa si el jugador pisa el bloque perimetral `0` o `15` con líneas naranjas.
* **Esquinas de Chunk**: Resalta las 4 esquinas exactas con marcadores magenta.
* **Rejilla 3D Completa**: Genera cajas delimitadoras alámbricas 3D en el motor Vulkan de Starling (desde Y=-64 hasta Y=320 y caja amarilla para el subchunk actual).

### Calculadora en Juego y Conversor de Stacks
* Ejecuta operaciones matemáticas instantáneas desde la barra o chat sin salir del juego:
  `/vui calc 128 * 4 + 64` ➔ `576 items = 9 stacks + 0 (shulkers: 0.33, cofres: 0.17)`.
* Cálculo de portales: `/vui calc portal 800 -1600` ➔ `Nether [100, -200]`.

### Visión Nocturna y Brújula de Recuperación
* **Visión Nocturna (`/vui nv` / `/vui gamma`)**: Controla el overlay de brillo en niveles de 25%, 50%, 75% y 100%.
* **Brújula de Recuperación (`/vui death`)**: Guarda automáticamente las coordenadas de tu última muerte y muestra en pantalla la distancia en metros en línea recta hacia tus objetos.

---

## 🗺️ VkMap: Motor de Mapa, Radar y Detección Subterránea

`VkMap` integra lo más potente y aclamado de los motores de mapa de Java (`Dynmap`, `Squaremap`, `VoxelMap`, `XaeroPlus`, `BlueMap`, `Conflux-Map`, `Diorama`) reescrito desde cero en Rust nativo optimizado para pantallas táctiles de Android y el motor Starling.

### 📐 3 Tamaños de Minimapa Táctil y Widgets Flotantes de Lupa
El mapa cuenta con tres modos de renderizado de pantalla conmutables al vuelo:
* **Pequeño (`96x96 px`)**: Modo ultra compacto para máxima visibilidad del juego sin estorbar los controles virtuales de movimiento.
* **Mediano (`144x144 px`)**: Tamaño equilibrado estándar nativo de Starling con radar y marcos de orientación cardinal.
* **Extra Grande / Pantalla Completa (`280x280 px`)**: Modo táctico expandido para navegación lejana, exploración de cuevas y visualización de ciudades o reclamos.

Al lado del mapa flotan 4 botones táctiles interactivos de acceso instantáneo:
* **`[ 🔍+ Ampliar ]`**: Pasa instantáneamente entre Pequeño ➔ Mediano ➔ Extra Grande.
* **`[ 🔍- Reducir ]`**: Reduce el tamaño del mapa.
* **`[ 🔎 Zoom: 1.0x ]`**: Alterna entre factores de aumento `0.5x`, `1.0x`, `2.0x` y `4.0x`.
* **`[ 🗺️ Menú ]`**: Abre el menú interactivo de 5 páginas con todas las opciones de navegación, radar, estructuras, perspectivas y exportación.

### 👁️ Modos de Visibilidad del Radar de Entidades
Para evitar saturar la pantalla con información redundante, el radar incluye 4 modos de visibilidad:
1. **`LineOfSight` (Línea de Visión)**: Solo muestra mobs o jugadores que estén en el cono frontal de visión del jugador (120°) y en un rango vertical de $\pm 12$ bloques, evitando que veas entidades invisibles detrás de paredes o enterradas en cuevas.
2. **`AllLayers` (Rayos-X Omnisciente)**: Muestra todas las entidades dentro del radio de alcance sin importar la altura o capa Y en la que se encuentren.
3. **`SurfaceOnly` (Solo Superficie)**: Filtra y muestra únicamente las entidades que caminan en tu misma superficie ($\pm 5$ bloques Y), ignorando totalmente las cavernas inferiores.
4. **`Fair-Play Mode` (Anti-Cheat)**: Oculta jugadores y hostiles en servidores competitivos multijugador para respetar las normativas de juego limpio.

### 🕳️ Analizador Subterráneo de Cuevas y Estructuras (Trial Chambers & Ancient Cities)
* Escanea automáticamente las capas subterráneas negativas ($Y = -64..0$).
* **Detección de Trial Chambers ($Y = -16$)**: Identifica la presencia de bloques icónicos como `minecraft:vault`, `minecraft:trial_spawner`, `minecraft:copper_bulb` y `minecraft:tuff_bricks`.
* **Detección de Ancient Cities ($Y = -48$)**: Localiza bloques característicos como `minecraft:reinforced_deepslate`, `minecraft:sculk_shrieker` y `minecraft:sculk_catalyst`.
* Modo de corte horizontal en vivo (`CaveMode { slice_y: -16, depth_blocks: 32 }`) para explorar túneles sin la interferencia del terreno superior.

### 📡 Sinergia Bidireccional con SeedCracker
* **Del Terreno al Cracking**: Cuando VkMap detecta estructuras o biomas en los chunks cargados, los transfiere en tiempo real a SeedCracker para acelerar el descifrado matemático de la semilla del mundo.
* **De la Semilla al Mapa**: Cuando SeedCracker calcula candidatos o descifra la semilla, proyecta en el mapa las estructuras matemáticamente predichas con el distintivo dorado **`[PRED]`**.
* **Promoción a Verificada**: Al acercarse el jugador y generarse los bloques en el mundo, VkMap promueve la estructura a verde **`[VERIF]`**.

### 🌍 Aislamiento de Sesiones por Mundo / Servidor y Guardado Automático
* Cada servidor multijugador o mundo individual guarda sus datos de manera totalmente aislada en `/sdcard/VkYakeeMods/VkMap/worlds/{world_id}/`:
  - `waypoints.json`: Waypoints tácticos, marcas de muerte y etiquetas de navegación.
  - `structures.json`: Estructuras descubiertas en el mundo y predichas por la semilla.
  - `annotations.json`: Polígonos de reclamos, carreteras y notas vectoriales de Conflux-Map.
* **Auto-guardado continuo**: Guarda el progreso cada 30 segundos y recarga de inmediato toda la exploración previa cuando el jugador vuelve a entrar a dicho servidor.

### 🖼️ Exportación de Mapas en PNG y JSON
* **Exportación Gráfica PNG (`/vkmap export png`)**: Renderiza y une en mosaico todos los chunks descubiertos en un archivo de imagen PNG de alta resolución guardado en `/sdcard/VkYakeeMods/VkMap/exports/`.
* **Exportación JSON (`/vkmap export json`)**: Genera un archivo `.json` completo con coordenadas, biomas, waypoints y estructuras estructuradas para cargarlo en visores web o calculadoras externas de semillas.

---

## 📊 Matriz de Paridad: HorizonMatica vs Litematica Original (Java)

| Característica / Función | Litematica Original (Java - masa) | Rustmatica (Rust) | HorizonMatica (VkYakeeMods Starling Android) | Estado en HorizonMatica |
| :--- | :---: | :---: | :---: | :---: |
| **Carga de Formato `.litematic`** | ✅ Nativo | ✅ Nativo | ✅ Nativo | 100% Paridad |
| **Carga de `.schem` (Sponge / WorldEdit)** | ✅ Soportado | ✅ Soportado | ✅ Soportado | 100% Paridad |
| **Carga de `.schematic` (MCEdit Clásico)** | ✅ Soportado | ✅ Soportado | ✅ Soportado | 100% Paridad |
| **Carga de `.nbt` (Vanilla Structure Blocks)** | ✅ Soportado | ✅ Soportado | ✅ Soportado | 100% Paridad |
| **Verificador de Bloques (`SchematicVerifier`)** | ✅ Menú Verifier | ✅ Algoritmo | ✅ HUD + `/hm verify` + Toasts interactivos | 100% Paridad |
| **Detección Missing / WrongBlock / WrongState** | ✅ Sí | ✅ Sí | ✅ Sí (Auditoría exacta bloque a bloque) | 100% Paridad |
| **Conteo de Materiales (Stacks / Shulkers)** | ✅ GUI Material List | ✅ Módulo | ✅ HUD + Stacks + Shulkers + Cofres | 100% Paridad |
| **Rastreo de Inventario y Faltantes** | ✅ Available vs Missing | ✅ `InventoryTracker` | ✅ `/hm mat missing` + HUD de faltantes | 100% Paridad |
| **Exportación de Materiales a Archivo** | ✅ Botón Export | ✅ `MaterialListExporter` | ✅ CSV y JSON en `/sdcard/VkYakeeMods/HorizonMatica/exports/` | 100% Paridad |
| **Rotación de Esquemática (90° CW / CCW)** | ✅ Sí | ✅ Sí | ✅ Touch button + `/hm rot cw\|ccw` | 100% Paridad |
| **Modo Espejo (Mirror X / Z)** | ✅ Sí | ✅ Sí | ✅ Touch button + `/hm mir` | 100% Paridad |
| **Bloqueo de Colocación (Placement Lock)** | ✅ Lock placement | ✅ Soportado | ✅ `[ 🔒 Bloquear ]` + `/hm lock` | 100% Paridad |
| **Corte de Capas Vertical (Eje Y)** | ✅ Layer Mode | ✅ Soportado | ✅ Touch buttons `[Capa ±]` + `/hm layer` | 100% Paridad |
| **Corte de Capas en Ejes X y Z (Multieje)** | ✅ Soportado | ✅ Soportado | ✅ `/hm layer axis [x\|y\|z]` + botón `[Eje]` | 100% Paridad |
| **Modos de Capa (Todas, Individual, Rango)** | ✅ All / Single / Range | ✅ `LayerMode` | ✅ Ciclo táctil `[Modo: Todas\|1\|Rango]` | 100% Paridad |
| **Control de Opacidad del Holograma** | ✅ Alpha slider | ✅ Alpha config | ✅ Ciclo táctil 25%, 50%, 75%, 100% + `/hm opacity` | 100% Paridad |
| **Protocolo EasyPlace (V2 / V3 / Slab)** | ✅ Auto / V2 / V3 | ✅ `EasyPlaceUtils` | ✅ `/hm ep protocol` + cálculo de click offset | 100% Paridad |
| **Creación de Esquemáticas (Pos1 / Pos2)** | ✅ Area Selection | ✅ BoxRegion | ✅ `[Pos1 Aquí]`, `[Pos2 Aquí]`, `/hm save` | 100% Paridad |
| **Caja Delimitadora de Selección (Visual 3D)** | ✅ Box render | ✅ BoundingBox | ✅ Bounding box Magenta en mundo 3D | 100% Paridad |
| **Proyectos y Versionado de Esquemáticas** | ✅ Schematic Projects | ✅ `SchematicProject` | ✅ `/hm project commit <nombre>` | 100% Paridad |
| **Previsualización en HUD Móvil** | ❌ (Solo en GUI compleja) | ❌ | ✅ **Mini-Render 3D Isométrico y 2D en HUD** | **Superado (Mejora Móvil)** |
| **Ergonomía Táctil Android (1-Botón Mini)** | ❌ (Diseñado para PC/Teclado) | ❌ | ✅ **Mini-HUD [🛠️ Herramientas] y Drawer** | **Superado (Mejora Móvil)** |

---

## 💾 Estructura del Almacenamiento Local en Android

Los datos se almacenan en una ruta pública accesible por cualquier explorador de archivos en Android (ZArchiver, Solid Explorer, MT Manager):

```text
/sdcard/VkYakeeMods/ (o ./VkYakeeMods_Data/ como fallback)
  │
  ├── SeedCracker/
  │     ├── worlds/    <- Sesiones guardadas por mundo o servidor (<world_id>.json)
  │     └── maps/      <- Mapas CSV exportados con biomas y estructuras (<world_id>_map.csv)
  │
  ├── HorizonMatica/
  │     ├── schematics/<- Archivos de esquemáticas (.litematic, .schem, .schematic, .nbt)
  │     └── exports/   <- Esquemáticas guardadas, proyectos y tablas de materiales (.csv, .json)
  │
  ├── VlakeeUI/
  │     └── assets/    <- Texturas, sonidos, definiciones e iconos originales de Déesse UI
  │
  └── VkMap/
        ├── worlds/    <- Subdirectorios aislados por mundo o servidor (<world_id>/)
        │     ├── waypoints.json    <- Puntos de muerte y waypoints guardados
        │     ├── structures.json   <- Estructuras verificadas [VERIF] y predichas [PRED]
        │     ├── annotations.json  <- Anotaciones vectoriales y reclamos
        │     └── tiles/            <- Teselas de mapa de chunks cacheadas en PNG
        └── exports/   <- Mosaicos PNG de alta resolución y datasets estructurados en JSON
```

---

## 📐 HorizonMatica: Flujo Interactivo y Herramientas

### ¿Qué botón uso para iniciar el holograma? (Flujo en 3 Pasos)

1. **Paso 1: `📁 [Cargar]` o `📥 [Importar]`**:
   * Selecciona una esquemática guardada o toca `[📥 Importar]` para detectar automáticamente archivos descargados en `/sdcard/Download/`.
2. **Paso 2: `📐 [Previsualizar]`**:
   * Observa el modelo en el **Mini-Renderizador 3D Isométrico** (con relieve de materiales) o en corte de capa 2D.
3. **Paso 3: `▶ [ INICIAR HOLOGRAMA ]` (Comando `/hm start`)**:
   * **Este es el botón de inicio**: Coloca la estructura en tus coordenadas de jugador.
   * **Cierra automáticamente el HUD grande** para no estorbar tu visión.
   * **Despliega el mini-HUD flotante de 1 botón**: **`[ 🛠️ Herramientas ]`**.

---

### Mini-HUD Flotante de 1 Botón: [🛠️ Herramientas]
Cuando el holograma está colocado y activo en el mundo:
```text
┌─ [ 🛠️ Herramientas: sample_shelter ] ──────────────┐
│ [ 🛠️ HERRAMIENTAS ] (Toca o /hm tools) | Rot: 90° | Capa Y: 64
│ Holograma: ON | Opacidad: 75% | LOCK: OFF | EasyPlace: ON (V2)
└──────────────────────────────────────────────┘
```
Tocar **`[ 🛠️ Herramientas ]`** abre el panel desplegable de edición.

---

### Panel de Herramientas y Edición
```text
┌─ [ 🛠️ Herramientas y Edición: HorizonMatica ] ───────────────┐
│ 🔄 [Rotar 90° CW/CCW] 🪞 [Espejo X/Z] 🔒 [🔓 LIBRE]
│ ⬆️ [Subir Y+1] ⬇️ [Bajar Y-1] ⬅️ [Mover X] ➡️ [Mover Z] ✨ [Op: 75%]
│ 📐 [Capa ±1] 👁️ [Modo: Todas] 📐 [Eje: Y] 🔨 [EasyPlace: ON]
├─ [ 🔍 Verificador y Materiales ] ────────────────────────────┤
│ 🔍 [Verificar Bloques] 📦 [Comprobar Inv] 📊 [Exportar CSV]  │
│ Verifier: 112 correctos, 0 faltantes, 0 errores
├─ [ ✏️ Creación de Nuevo Litematic ] ─────────────────────────┤
│ 📍 [Pos 1: Aquí] 📍 [Pos 2: Aquí] 💾 [Guardar Selección]      │
│ Selección: [10,64,10] ➔ [20,74,20] (1331 blqs)
├─ [ 🔘 Opciones de Pantalla ] ────────────────────────────────┤
│ 🔽 [Minimizar a 1 Botón] (/hm tools) | 📋 [Menú Principal]   │
└─ (/hm tools | /hm verify | /hm mat | /hm up | /hm down | /hm pos1) ┘
```

* **Bloqueo de Colocación (`[ 🔒 Bloquear ]` / `/hm lock`)**: Evita mover accidentalmente el esquema mientras colocas bloques.
* **Giro y Espejo**: Rotación horaria/antihoraria y reflejo horizontal.
* **Ajuste de Opacidad**: Atenuación dinámica (25%, 50%, 75%, 100%) para ver tanto el holograma como los bloques colocados.
* **Auditoría con el Verificador de Esquemáticas (`SchematicVerifier`)**: Detecta bloques faltantes (`Missing`), bloques erróneos (`WrongBlock`) o propiedades invertidas (`WrongState`).
* **Exportación de Tablas (`MaterialListExporter`)**: Genera reportes en CSV y JSON directamente a `HorizonMatica/exports/`.

---

## 🔬 SeedCracker en Rust: Guía y Algoritmos

SeedCracker reconstruye la **semilla de mundo de 64 bits** ($2^{64}$ combinaciones) sin permisos de operador (`/seed`):
1. **Inversión del Generador Lineal Congruencial (Java LCG) de 48 bits**:
   $$X_{n+1} = (X_n \times a + c) \pmod{m}$$
   Con multiplicador inverso modular $a^{-1} = 0xDFE05BCB1365 \pmod{2^{48}}$, revierte llamadas con complejidad $O(1)$.
2. **Reducción de Retículos LLL y Babai CVP**:
   Modelamos la correlación entre estructuras como un retículo geométrico multidimensional. Aplicando reducción de bases **Lenstra-Lenstra-Lovász (LLL)** y **Babai Closest Vector Problem (CVP)**, se reduce el espacio de $2^{48}$ posibilidades a un puñado de semillas candidatas en **0.12 milisegundos**.
3. **Cracker Jerárquico de Bedrock Bit-Slice**:
   Árbol de decisión de 12 niveles de capas de Bedrock que extrae los 48 bits inferiores en **< 25 ms**.
4. **Base de Datos Precomputada de Tesoros Enterrados**:
   10 archivos embebidos en el binario que resuelven semillas a partir de cofres de tesoro en costas en $O(1)$.
5. **Motor de Biomas y Estructuras C Cubiomes**:
   Compilado nativamente con optimizaciones `-O3` para biomas y estructuras de Minecraft 1.18 a 1.21+.
6. **Sistema Híbrido de Descarte Predictivo (`SeedVerifier`)**:
   Fase matemática hasta $\le 2000$ candidatos y fase en terreno con waypoints asistidos.

---

## 📱 Ergonomía Táctil y Prevención de Solapamiento en Android

* **Zona Segura Superior Izquierda**: El distintivo **`[ ⚡ VkMods ]`** y el mini-HUD flotante se sitúan en $x = 10\text{ px}, y = 4\text{ px}$, dejando **más de 250 píxeles libres** respecto al joystick virtual de movimiento (esquina inferior izquierda) y alejados de los botones de salto, ataque y agachado (derecha).
* **Pestañas Rápidas de HUD**:
  * `/vk tab vui`: Muestra exclusivamente **VlakeeUI** (SmartF3, InvHUD, Durabilidad).
  * `/vk tab hm`: Muestra exclusivamente **HorizonMatica** (Esquemáticas y Herramientas).
  * `/vk tab sc`: Muestra exclusivamente **SeedCracker** (Semilla y Chunks).
  * `/vk tab all`: Muestra todos los mods simultáneamente.

---

## 🛡️ Seguridad y Anti-Detección en Servidores (100% Client-Side Stealth)

1. **Intercepción Local de Comandos**: `/vui`, `/f3`, `/hm`, `/sc` y `/vk` son interceptados antes del socket de red; **ningún paquete viaja al servidor**.
2. **Cero Fugas en el Chat**: Toda la información se renderiza en la capa local de HUD del motor gráfico Vulkan.
3. **Escucha Pasiva**: Sin envíos de paquetes extraños ni desincronización de ticks.

---

## 🎮 Comandos y Accesos Rápidos

### Comandos de VlakeeUI
| Comando | Descripción |
| :--- | :--- |
| `/vk f3off` o `/vui f3mute` | **F3 Maestro**: Apaga o enciende simultáneamente SmartF3 y el F3 nativo de Starling. |
| `/vui more` | Abre la **Página 4 (SmartF3 More)** con los atajos técnicos y de speedrunner. |
| `/f1` o `/vui f1` | **Modo F1**: Oculta por completo la interfaz, HUD y barra de objetos (cinemática/screenshot). |
| `/vui f3 b` o `/vui f3b` | **F3+B**: Alterna las hitboxes de colisión de entidades (vital para speedrunners). |
| `/vui f3 g` o `/vui f3g` | **F3+G**: Muestra u oculta los límites y rejilla 3D de chunks. |
| `/vui f3 h` o `/vui f3h` | **F3+H**: Alterna los tooltips avanzados (durabilidad numérica, IDs y NBT count). |
| `/vui f3 n` o `/vui f3n` | **F3+N**: Alterna rápidamente al Modo Espectador y vuelve al modo previo. |
| `/vui f3 a` o `/vui f3a` | **F3+A**: Fuerza la recarga y re-renderizado de todos los chunks. |
| `/vui f3 c` o `/vui f3c` | **F3+C**: Copia el comando exacto de teletransporte a tus coordenadas actuales. |
| `/vui f3 d` o `/vui f3d` | **F3+D**: Limpia el historial de mensajes del chat. |
| `/vui f3 t` o `/vui f3t` | **F3+T**: Recarga paquetes de recursos y texturas. |
| `/vui pie` o `/vui profiler` | **Shift+F3**: Muestra el Profiler / Pie Chart de rendimiento y conteo de entidades. |
| `/vui craft` | Abre la **Página 5 (Crafteo Rápido / Mass Craft)**. |
| `/vui craft mode` | Alterna el modo de crafteo entre Normal, Mass Craft (todo el inventario) y 1-Stack. |
| `/vui craft [planks\|sticks\|table\|torches\|blocks\|ingots\|rockets\|bread]` | Craftea instantáneamente la receta seleccionada con 1 comando. |
| `/vui f3 [compact\|full\|off]` | Alterna o fija el modo de SmartF3 / Telemetría. |
| `/f3` | Atajo rápido para alternar SmartF3 de inmediato. |
| `/f5` o `/vui cam [first\|back\|front]` | **Cámara F5**: Alterna entre 1ª persona, 3ª persona espalda y 3ª persona frontal. |
| `/vui gm [s\|c\|sp\|a\|next]` | **Modo de Juego**: Cambia a Supervivencia, Creativo, Espectador o Aventura. |
| `/vui fov [30\|75\|110\|cycle]` | **FOV Rápido**: Alterna presets de Zoom (30°), Normal (75.6°) o Quake Pro (110°). |
| `/vui render [8\|12\|16\|24]` | **Render Rápido**: Ajusta la distancia de renderizado de chunks en 1 paso. |
| `/vui time [day\|noon\|night]` | **Tiempo**: Fija la hora del mundo a Día, Mediodía, Noche o Medianoche. |
| `/vui weather [clear\|rain\|thunder]` | **Clima**: Cambia a clima Despejado, Lluvia o Tormenta eléctrica. |
| `/vui note <texto>` | **Bloc de Notas**: Agrega una nota temporal en juego. |
| `/vui notes` / `/vui clearnotes` | Muestra o borra las notas temporales activas. |
| `/vui mark <nombre>` | **Waypoint**: Guarda tus coordenadas actuales con un nombre identificador. |
| `/vui waypoints` / `/vui delmark <id>` | Lista o elimina waypoints guardados. |
| `/vui sharepos` o `/vui pastecoords` | Formatea tus coordenadas en un mensaje listo para enviar al chat. |
| `/vui container [move\|drop\|restock]` | **Atajos Contenedor**: Mover todo a cofre, tirar todo o reabastecer hotbar. |
| `/vui pause` | Despliega u oculta el cajón de opciones del menú de pausa en juego. |
| `/vui nether` | Alterna el convertidor de coordenadas Overworld <-> Nether. |
| `/vui chunk [mid\|edge\|corner\|grid]` | Alterna los indicadores de centro, bordes, esquinas o rejilla 3D de chunks. |
| `/vui inv` | Activa o desactiva la vista rápida de inventario (InvHUD). |
| `/vui dura` | Muestra u oculta la durabilidad de herramientas y armadura. |
| `/vui nv [25\|50\|75\|100]` | Activa, desactiva o ajusta el brillo de visión nocturna. |
| `/vui clock` | Muestra u oculta el reloj solar y brújula de dirección. |
| `/vui hidecoords` | Oculta o muestra tus coordenadas en la pantalla. |
| `/vui hidechat` | Oculta el panel de chat en pantalla. |
| `/vui hidesb` | Oculta la barra lateral de scoreboard. |
| `/vui death` | Muestra o fija las coordenadas de tu última muerte (Recovery Compass). |
| `/vui calc <operación>` | Realiza cálculos matemáticos y conversiones a stacks en el chat. |
| `/vui page` | Navega cíclicamente entre las 5 páginas de botones táctiles del HUD. |

### Comandos de HorizonMatica
| Comando | Descripción |
| :--- | :--- |
| `/hm start` o `/hm iniciar` | **Iniciar**: Coloca el holograma en tus coords, minimiza el HUD y abre el mini-HUD de herramientas. |
| `/hm tools` o `/hm herramientas` | Alterna entre el mini-HUD de 1 botón y el panel desplegable de herramientas. |
| `/hm menu` o `/hm main` | Abre el menú principal con importación y previsualización. |
| `/hm v` o `/hm verify` | **Verificador**: Compara la construcción con la esquemática y detecta bloques erróneos o faltantes. |
| `/hm mat` | Muestra lista de materiales, totales, stacks y shulkers. |
| `/hm mat missing` | Calcula materiales disponibles en tu inventario vs items faltantes. |
| `/hm mat export [csv\|json]` | Exporta la tabla de materiales a archivo en `HorizonMatica/exports/`. |
| `/hm lock` | **Bloquear/Desbloquear**: Fija la colocación para prevenir movimientos accidentales. |
| `/hm up` / `/hm down` | Sube o baja el holograma 1 bloque (Y±1). |
| `/hm left` / `/hm right` | Mueve el holograma en el eje lateral (X±1). |
| `/hm rot [cw\|ccw]` | Rota la esquemática 90° en sentido horario o antihorario. |
| `/hm mir` | Alterna modo espejo (Ninguno, X, Z). |
| `/hm layer <up\|down\|all\|y>` | Navega entre capas de construcción o muestra todas. |
| `/hm layer axis [x\|y\|z]` | Cambia el eje de corte de capas entre Y (vertical), X o Z. |
| `/hm opacity [25\|50\|75\|100]` | Modifica la transparencia del holograma. |
| `/hm ep` | Activa o desactiva el modo EasyPlace. |
| `/hm ep protocol [v2\|v3\|auto\|slab]` | Cambia el protocolo de colocación y orientación de EasyPlace. |
| `/hm pos1` / `/hm pos2` | Fija esquinas 1 y 2 para crear una nueva esquemática. |
| `/hm save <nombre>` | Guarda la selección como nuevo `.litematic`. |
| `/hm project commit <nombre>` | Guarda un commit de versión en el gestor de proyectos. |
| `/hm import [archivo]` | Importa esquemáticas o busca en Descargas. |
| `/hm load <nombre_o_indice>` | Carga una esquemática guardada en Android. |
| `/hm unload` | Descarga la esquemática y oculta el holograma. |

### Comandos de VkMap
| Comando | Descripción |
| :--- | :--- |
| `/vkmap` o `/map` | Muestra u oculta el minimapa en pantalla. |
| `/vkmap hud` | Activa o desactiva la barra de telemetría y estado de VkMap. |
| `/vkmap size [small\|medium\|fullscreen\|cycle]` | Cambia el tamaño del mapa (Pequeño 96px, Mediano 144px, Pantalla Completa 280px). |
| `/vkmap zoom [in\|out\|0.5\|1.0\|2.0\|4.0]` | Modifica el nivel de aumento del mapa. |
| `/vkmap radar [los\|all\|surface]` | Cambia el modo de visibilidad del radar (Línea de visión, Rayos X, Superficie). |
| `/vkmap cave [on\|off\|<capa_y>]` | Activa o desactiva el analizador subterráneo de cuevas (Trial Chambers y Ancient Cities). |
| `/vkmap wp add <nombre>` | Crea un nuevo waypoint táctico en tu ubicación actual. |
| `/vkmap wp death` | Muestra las coordenadas de tu último punto de muerte registrado. |
| `/vkmap export [png\|json]` | Exporta el mapa completo a imagen PNG o dataset estructurado a JSON en `/sdcard/VkYakeeMods/VkMap/exports/`. |
| `/vkmap world <id>` | Cambia el contexto de mundo/servidor y recarga sus datos de forma aislada. |
| `/vkmap save` | Fuerza el guardado manual de los datos del mundo actual. |
| `/vk tab map` | Cambia la pestaña activa del HUD global de VkYakeeMods a VkMap. |

---

## ⚖️ Créditos y Licencias

### Reescritura
* **Reescrito en Rust para Starling (Horizon Android) por: `VkYakee`**

### Proyectos y Autores Originales:
* **Dynmap**: Renderizador isométrico, curvas de luz y rebanado de cuevas por **mikeprimm**.
* **Squaremap & Pl3xMap**: Raycasting de ultra-alto rendimiento y relieve 3D por **jmp** y **BillyGalbreath**.
* **VoxelMap**: HUD de minimapa, stencils y radar de entidades con indicadores de elevación por **MamiyaOtaru**.
* **Xaero's Minimap & XaeroPlus**: Aislamiento por servidor/mundo, reglas Fair-Play y conversión 1:8 de portales por **chylex** y **Xaero96**.
* **BlueMap**: Generación de jerarquías LOD por **Bluecolored**.
* **Conflux-Map**: Anotaciones vectoriales, caminos y reclamos por **Conflux**.
* **Déesse UI (Bedrock Edition)**: Creado originalmente por **@Itzriyo157**, con aportes de **@T1M0THY**, **@Dimzcraft**, **@tlgm2308**, **The Andromedarius** y **drav0011**.
* **Litematica**: Creado originalmente en Java por **masa**.
* **Rustmatica**: Implementación de referencia en Rust por la comunidad open-source.
* **SeedCracker**: Creado originalmente por **KaptainWutax**.
* **SeedCrackerX**: Mantenido y extendido por **19MisterX98**.
* **cubiomes**: Biblioteca C de biomas y generadores de ruido por **Protupit**.
* **NetherBedrockCracker**: Algoritmo de Bedrock por **Neil**.
* **TreeCracker**: Algoritmos de árboles por **Cortex**.
* **SeedMapper**: Formaciones de tesoros por **xpple**.

### Términos de Licencia y Modificación
Este software se distribuye bajo la licencia **MIT**:
* Se permite el uso libre, estudio, modificación, integración en modpacks y redistribución comercial o privada.
* **Condición obligatoria**: Toda redistribución o modificación de este código debe conservar los créditos de los autores originales citados anteriormente y la atribución: *"Este mod fue Reescrito en Rust por VkYakee"*.

---

*Desarrollado con pasión para la comunidad de Minecraft Rust en Android.*
