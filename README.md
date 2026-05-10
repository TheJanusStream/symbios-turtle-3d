# symbios-turtle-3d

A spatial interpretation layer for [Symbios](https://crates.io/crates/symbios) L-Systems using [glam](https://crates.io/crates/glam) for 3D math.

This crate provides a 3D turtle graphics interpreter that converts L-System symbol sequences into geometric skeletons suitable for mesh generation.

## Features

- **Standard L-System operations**: Draw (`F`), Move (`f`), rotations (`+`, `-`, `&`, `^`, `\`, `/`), branching (`[`, `]`)
- **Palette-based materials**: Color, material ID, and UV scale per segment — roughness, metallic, and other PBR properties are defined externally via a material palette
- **Tropism support**: Configurable gravity/light attraction for natural plant growth
- **Prop spawning**: Place discrete objects (leaves, flowers) with the `~` operator
- **Branch hierarchy**: `Skeleton::strand_parents` records which strand each branch sprouts from, so renderers can distinguish trunks from limbs without re-running the turtle
- **Bounded stack depth**: `TurtleConfig::max_stack_depth` caps `[` recursion; the matching `]` is also skipped on overflow and a `TurtleWarning::StackOverflow` is recorded on `Skeleton::warnings` rather than corrupting downstream geometry

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
symbios-turtle-3d = "0.5"
symbios = "1.5"
glam = "0.30.10"
```

## Usage

```rust
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

// Builder-style setup: interns the canonical turtle alphabet
// (`F`, `f`, `+`, `-`, `&`, `^`, `\`, `/`, `|`, `$`, `!`, `[`, `]`, `~`, `'`, `,`, `;`)
// and registers each one's default operation in a single call.
let mut interner = SymbolTable::new();
let interpreter = TurtleInterpreter::new(TurtleConfig::default())
    .with_standard_symbols(&mut interner);

// Build an L-System state (normally from symbios expansion)
let mut state = SymbiosState::new();
let f_id = interner.resolve_id("F").unwrap();
state.push(f_id, 0.0, &[10.0]).unwrap(); // F(10)

// Interpret to skeleton
let skeleton = interpreter.build_skeleton(&state);

// Walk strands and their branch parentage for mesh generation.
for (i, strand) in skeleton.strands.iter().enumerate() {
    let parent = skeleton.strand_parents[i];
    println!("strand {i} parent={parent:?}");
    for point in strand {
        println!("  pos: {:?}, radius: {}", point.position, point.radius);
    }
}

// Surface non-fatal interpretation issues (e.g. stack-depth overflow).
for w in &skeleton.warnings {
    eprintln!("warning: {w:?}");
}
```

If you'd rather intern symbols yourself (e.g. to mix in non-standard symbols),
use `populate_standard_symbols(&interner)` after interning instead of the
`with_standard_symbols` builder — only symbols already present in the symbol
table are wired up; un-interned standard symbols are silently skipped.

## Configuration

```rust
use symbios_turtle_3d::TurtleConfig;
use glam::Vec3;

let config = TurtleConfig {
    default_step: 1.0,                    // Default length for F/f
    default_angle: 45.0_f32.to_radians(), // Default rotation angle
    initial_width: 0.1,                   // Starting stroke width
    tropism: Some(-Vec3::Y),              // Gravity direction (None disables tropism)
    elasticity: 0.2,                      // How strongly the turtle bends toward tropism (0.0 = off)
    max_stack_depth: 1024,                // Cap on `[` nesting; overflow records a TurtleWarning
};
```

For partial overrides, spread `TurtleConfig::default()`:

```rust
# use symbios_turtle_3d::TurtleConfig;
# use glam::Vec3;
let config = TurtleConfig {
    tropism: Some(-Vec3::Y),
    elasticity: 0.2,
    ..TurtleConfig::default()
};
```

## Material Philosophy: Substance vs. Variation

This crate follows a **palette-first** approach to materials. Instead of specifying PBR properties
(roughness, metallic, etc.) per-segment in the L-System grammar, these properties are defined
externally in a **material palette** indexed by `material_id`.

The L-System grammar controls:

- **Color** (`'`) — per-segment albedo tint
- **Material ID** (`,`) — selects a palette entry that defines the full PBR substance
- **UV Scale** (`;`) — adjusts texture density without changing the substance

This separation keeps grammars focused on *what varies* (color, which material, texture density)
while the palette handles *what stays consistent* (roughness, metallic, normal maps).

## Symbol Reference

All parameters are optional. When omitted, length and angles fall back to
`TurtleConfig::default_step` / `default_angle`; `!`, `;` fall back to the
turtle's current width / UV scale; `~` defaults to prop id 0 with unit scale.

| Symbol    | Operation              | Parameters                                                        |
|-----------|------------------------|-------------------------------------------------------------------|
| `F`       | Draw forward           | `(length)`                                                        |
| `f`       | Move forward (no draw) | `(length)`                                                        |
| `+` / `-` | Yaw (rotate Z)         | `(angle°)`                                                        |
| `&` / `^` | Pitch (rotate X)       | `(angle°)`                                                        |
| `\` / `/` | Roll (rotate Y)        | `(angle°)`                                                        |
| `\|`      | Turn around (180°)     | -                                                                 |
| `$`       | Align to vertical      | -                                                                 |
| `!`       | Set width              | `(width)`                                                         |
| `[` / `]` | Push/Pop state         | -                                                                 |
| `~`       | Spawn prop             | `()`, `(prop_id)`, `(prop_id, scale)`, or `(prop_id, sx, sy, sz)` |
| `'`       | Set color              | `(gray)` or `(r,g,b)` or `(r,g,b,a)`                              |
| `,`       | Set material ID        | `(id)`                                                            |
| `;`       | Set UV scale           | `(scale)`                                                         |

### Turtle frame conventions

The turtle moves head-first along its local **+Y** axis. Rotations are applied
in the turtle's local frame:

- Yaw (`+`/`-`) rotates around local Z — the turtle's "up" axis
- Pitch (`&`/`^`) rotates around local X — the turtle's right axis
- Roll (`\`/`/`) rotates around local Y — the turtle's heading axis

## Ecosystem

```text
symbios (derivation engine)
  └── symbios-turtle-3d (3D interpreter)  ← you are here
        └── bevy_symbios (Bevy meshes, materials, export, UI)
              └── lsystem-explorer (interactive application)
```

## License

MIT
