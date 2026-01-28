# symbios-turtle-3d

A spatial interpretation layer for [Symbios](https://crates.io/crates/symbios) L-Systems using [glam](https://crates.io/crates/glam) for 3D math.

This crate provides a 3D turtle graphics interpreter that converts L-System symbol sequences into geometric skeletons suitable for mesh generation.

## Features

- **Standard L-System operations**: Draw (`F`), Move (`f`), rotations (`+`, `-`, `&`, `^`, `\`, `/`), branching (`[`, `]`)
- **Palette-based materials**: Color, material ID, and UV scale per segment — roughness, metallic, and other PBR properties are defined externally via a material palette
- **Tropism support**: Configurable gravity/light attraction for natural plant growth
- **Prop spawning**: Place discrete objects (leaves, flowers) with the `~` operator

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
symbios-turtle-3d = "0.1"
symbios = "1.0"
glam = "0.30"
```

## Usage

```rust
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

// Create symbol table and interpreter
let mut interner = SymbolTable::new();
let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

// Intern the symbols you'll use
interner.intern("F").unwrap();
interner.intern("+").unwrap();
interner.intern("[").unwrap();
interner.intern("]").unwrap();

// Populate standard symbol-to-operation mappings
interpreter.populate_standard_symbols(&interner);

// Build an L-System state (normally from symbios expansion)
let mut state = SymbiosState::new();
let f_id = interner.resolve_id("F").unwrap();
state.push(f_id, 0.0, &[10.0]).unwrap(); // F(10)

// Interpret to skeleton
let skeleton = interpreter.build_skeleton(&state);

// Use skeleton.strands and skeleton.props for mesh generation
for strand in &skeleton.strands {
    for point in strand {
        println!("pos: {:?}, radius: {}", point.position, point.radius);
    }
}
```

## Configuration

```rust
use symbios_turtle_3d::TurtleConfig;
use glam::Vec3;

let config = TurtleConfig {
    default_step: 1.0,                    // Default length for F/f
    default_angle: 45.0_f32.to_radians(), // Default rotation angle
    initial_width: 0.1,                   // Starting stroke width
    tropism: Some(-Vec3::Y),              // Gravity direction
    elasticity: 0.2,                      // How much turtle bends toward tropism
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

| Symbol | Operation | Parameters |
|--------|-----------|------------|
| `F` | Draw forward | `(length)` |
| `f` | Move forward (no draw) | `(length)` |
| `+` / `-` | Yaw (rotate Z) | `(angle°)` |
| `&` / `^` | Pitch (rotate X) | `(angle°)` |
| `\` / `/` | Roll (rotate Y) | `(angle°)` |
| `\|` | Turn around (180°) | - |
| `$` | Align to vertical | - |
| `!` | Set width | `(width)` |
| `[` / `]` | Push/Pop state | - |
| `~` | Spawn prop | `(prop_id, scale)` |
| `'` | Set color | `(gray)` or `(r,g,b)` or `(r,g,b,a)` |
| `,` | Set material ID | `(id)` |
| `;` | Set UV scale | `(scale)` |

## Ecosystem

```
symbios (derivation engine)
  └── symbios-turtle-3d (3D interpreter)  ← you are here
        └── bevy_symbios (Bevy meshes, materials, export, UI)
              └── lsystem-explorer (interactive application)
```

## License

MIT
