# symbios-turtle-3d

A spatial interpretation layer for [Symbios](https://crates.io/crates/symbios) L-Systems using [glam](https://crates.io/crates/glam) for 3D math.

This crate provides a 3D turtle graphics interpreter that converts L-System symbol sequences into geometric skeletons suitable for mesh generation.

## Features

- **Standard L-System operations**: Draw (`F`), Move (`f`), rotations (`+`, `-`, `&`, `^`, `\`, `/`), branching (`[`, `]`)
- **PBR material properties**: Color, roughness, metallic, texture ID per segment
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
| `~` | Spawn prop | `(surface_id, scale)` |
| `'` | Set color | `(gray)` or `(r,g,b)` or `(r,g,b,a)` |
| `,` | Set material ID | `(id)` |
| `#` | Set roughness | `(0.0-1.0)` |
| `@` | Set metallic | `(0.0-1.0)` |
| `;` | Set texture ID | `(id)` |

## License

MIT
