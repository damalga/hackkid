# Hackkid — Proyección del juego

Motor terminal Rust + raycaster estilo Wolfenstein 3D. Renderizado con half-block chars (`▄`) sobre crossterm.

## Estado narrativo actual

- Despiertas en un hospital abandonado sin memoria clara
- Único NPC hasta ahora: **Samuel**, en el hub central
- Diálogo lineal con Samuel te da el mapa del hospital tras 7 líneas
- Se menciona a **Selenia**, otra superviviente, ahora fuera del hospital (no implementada aún)

## Bucle jugable

- Explorar hospital en primera persona
- Descansar (sentarse/tumbarse) para regenerar stats
- Recoger objetos y ropa
- Hablar con NPCs
- Usar objetos (portátil abre vista de laptop)

## Controles

| Tecla | Acción |
|-------|--------|
| W / S | Avanzar / retroceder |
| A / D | Girar |
| Rueda ratón | Correr (2× velocidad avanzar/retroceder) |
| X | Interactuar (recoger, dialogar, sentar, tumbar, abrir puerta) |
| I | Abrir mochila (bloquea movimiento) |
| ↑↓ (mochila abierta) | Navegar slot |
| Enter (mochila abierta) | Usar objeto |
| T (mochila abierta) | Tirar objeto |
| Esc / P (laptop) | Apagar |
| H | Ocultarse |
| Enter | Menú principal (controles) |
| Ctrl+S | Guardar partida (`save.json`) |
| Ctrl+L | Cargar partida (`save.json`) |
| Ctrl+C | Salir |

## Mapa del hospital (45×46)

```
Origin ward  (x=10..34, y=0..12)   4 habitaciones patient
  |
Hub          (x=10..34, y=12..23)  sala principal, Samuel
  |
  ├── Bathrooms públicos (x=2..10, y=17..22)
  ├── Right ward (x=34..44, y=12..23) rooms A/B + main hall
  └── Pasillo largo (x=20..24, y=23..44) tras 2ª main door
```

### Habitaciones origin

Cada una **5×5 tiles** con:
- Baño sub-room 3×2 en esquina NE (WC + lavabo + ducha)
- Puerta baño gris clarito con letrero WC
- Sala de estar con **cama (SW)**, **sofá (SE)**, **perchero (NW)**
- Puerta al pasillo en el sur, alineada con centro habitación

### Puertas

- Marrón claro (`Door`) → puertas de habitación
- Marrón oscuro (`MainDoor`) → puertas principales, doble hoja al abrir
- Gris claro (`BathroomDoor`) → baños

Letreros diferenciales al lado (misma pared que puerta):
- `RoomSmall` marrón claro → hab
- `MainSquare` azul → main door
- `Bathroom` cuadrado grande con "WC" pintado → baño
- `HallwayLong` rectángulo dorado → puerta al pasillo largo

## HUD (4 cajas horizontales, última libre)

```
[ Salud ][ Mapa ][ Inventario (I abrir) ][ Vestuario ]
```

### Salud
Día + hora + 7 barras: Cuerpo, Mente, Stamina, Sed, Hambre, Sueño, Térmica (bipolar frío|calor).

### Mapa
- Ventana centrada en jugador (siempre en el centro)
- Paredes `·` blancas, puertas `+` amarillas, jugador `●` amarillo
- Sólo visible tras recibir mapa de Samuel; si no: "No tienes el mapa de este lugar"

### Inventario
- 6 slots verticales `▶ label`
- Se abre con `I` (bloquea walk)
- Sólo si tienes mochila

### Vestuario
- Prendas equipadas (inicial: "Camisón de hospital")

## Stats

| Stat | Rango | Decaimiento |
|------|-------|-------------|
| Cuerpo | 0-100 | -0.002/tick |
| Mente | 0-100 | -0.001/tick |
| Stamina | 0-100 | regen si no corres |
| Sed | 0-100 | -0.004/tick |
| Hambre | 0-100 | -0.003/tick |
| Sueño | 0-100 | -0.003/tick |
| Térmica | -100..+100 | tiende a 0 al descansar |

### Descanso

- **Sentado** (banco/sofá, X): +0.08 sueño, +0.12 stamina, +0.02 cuerpo/mente/tick
- **Tumbado** (cama, X): +0.20 sueño/stamina, +0.05 cuerpo/mente/tick
- Vista sentado = 3D normal fijo; tumbado = techo

## Mochila y objetos

- Slot capacity **6**. Riñonera futura ampliará capacidad
- Objetos iniciales al recoger mochila: **Portátil**, **Fuente de alimentación**, **Cantimplora**
- Ropa (`Hoodie`, `Pants`) requiere mochila para recoger
- Cualquier objeto se tira con `T` al suelo (dropped item, recogible)

### Portátil
- Enter en slot con "Portátil" → vista laptop
- Interfaz: icono `Navegador` arriba izquierda, `Batería XX%` abajo derecha
- Movimiento bloqueado
- Esc/P apaga

### Batería y carga

- Batería inicial 62%, drena -0.05/tick cuando abierto
- Cerca de un `Outlet` (< 1.5 tiles) → **+0.10/tick, indicador `⚡ cargando`**
- Se apaga automáticamente al llegar a 0

## Objetos del mundo

| Tipo | Interacción |
|------|-------------|
| Cama | X = tumbarse |
| Sofá | X = sentarse (púrpura, tapizado con reposabrazos) |
| Banco | X = sentarse (madera, pegado a pared) |
| Perchero | Decorativo (sudadera colgada en room 3) |
| Enchufe | Fuente de alimentación (junto a asientos y camas) |
| Baño (WC, lavabo, ducha, urinario) | Decorativo (privado 3 fixtures, público 2 sinks + 2 WC + 1 urinario) |
| NPC | X = dialogar |
| Fluorescente | Iluminación de techo, algunos parpadean o están muertos |

## Diálogo con Samuel (secuencial con X)

1. Tú: "Hola, ¿quién eres?"
2. Samuel: presentación (Me llamo Samuel...)
3. Tú: "... no entiendo nada."
4. Samuel: menciona a Selenia
5. Tú: "Espera... ¿y dónde está el baño?"
6. Samuel: te ofrece el mapa
7. "Has recibido el mapa del hospital. Pulsa Q para verlo."
8. → futuras interacciones: `Samuel: '...'`

Se concede `has_map = true` al llegar al final del script.

## Rendering técnico

- Raycaster DDA sobre `Tile { Floor, Wall, Door, MainDoor, BathroomDoor, PushBox }`
- Sprites billboards con u/v mapping (transparencia via `Option<(u8,u8,u8)>`)
- Iluminación por segmentos de fluorescente (distancia a segmento)
- Half-block char per row → 2 pixels verticales / celda terminal
- Overlay:
  - Border top línea
  - Message box con borde (Tú:/Samuel:/genéricos)
  - Prompt centrado `X: <acción>` cuando hay acción disponible
  - Menú principal centrado (Enter)

## Persistencia

- `Ctrl+S` serializa a `save.json`
- `Ctrl+L` carga `save.json`
- Guarda: posición/dir/plano jugador, turn/day/hour, stats, inventario tipado, wardrobe, dropped items, samuel_stage+line, has_map, has_backpack, hidden, mode, laptop_battery+cursor, estado de puertas (open/closed), equippables tomados, clothing tomado

## Arquitectura del código

```
src/
├── main.rs              # bootstrap + game loop + collect_sprites
├── engine/
│   ├── map.rs           # Tile, Map, stamp_* geometry
│   ├── raycaster.rs     # DDA + Ray
│   ├── renderer.rs      # build_frame, draw_ceiling_lights, draw_sprites
│   └── sprites.rs       # SpriteShape enum + pixel functions
├── game/
│   ├── action.rs        # Action enum + dispatch (mode-gated)
│   ├── input.rs         # key/mouse → Action mapping
│   ├── items.rs         # Item + ItemKind (Laptop, PowerSupply, Canteen, Hoodie...)
│   ├── world.rs         # World, dialogue, pickup, save/load
│   └── player.rs        # Player, Stats, Inventory (Vec<Option<Item>>)
├── objects/
│   ├── bed.rs
│   ├── bench.rs
│   ├── clothing.rs      # Hoodie, Pants, HospitalGown, ScrubBundle
│   ├── coat_rack.rs
│   ├── door.rs          # DoorKind: Regular, Main, Bathroom
│   ├── fixture.rs       # Toilet, Sink, Shower, Urinal + componentes reutilizables
│   ├── fluorescent.rs   # steady, flicker, dead
│   ├── npc.rs
│   ├── outlet.rs
│   ├── sign.rs
│   └── sofa.rs
├── equippables/
│   └── backpack.rs
└── ui/
    ├── ceiling.rs       # render_ceiling (wake / lying view)
    ├── frame.rs         # blit_frame, top border, pixel helper, constants
    ├── hud.rs           # stat_bar, stat_bar_center
    ├── hud_area.rs      # 4-box HUD (Salud / Mapa / Inventario / Vestuario)
    ├── laptop.rs        # render_laptop_view
    ├── menu.rs          # render_main_menu
    ├── overlay.rs       # message overlay + action prompt
    ├── prompt.rs        # build_action_label (context X: <...>)
    └── widgets.rs       # draw_box helper
```

## Componentes reutilizables

- `private_bathroom(x, y)` → toilet + sink + shower separados 1 tile
- `public_bathroom(x, y)` → 2 sinks + 2 toilets + 1 urinal
- `draw_box(x, y, w, h, title)` → caja bordeada con título ajustable
- `render_action_prompt(action)` → caja centrada `X: Acción`

## Próximos pasos posibles

1. **Riñonera** para ampliar inventario (mencionada en diseño)
2. **Selenia** NPC fuera del hospital + arco narrativo
3. **Baños en right ward** con walls (actualmente sin sub-room)
4. **Poblado del pasillo largo** (aún vacío intencionalmente)
5. **Uso real de items**: cantimplora → sed, comida → hambre
6. **Consumo laptop diferenciado**: apps consumen distinto
7. **Menú principal antes de partida** con Continuar/Nueva/Salir
8. **Braille chars** para mayor resolución de píxel si se busca definición
