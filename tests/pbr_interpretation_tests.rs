use approx::assert_relative_eq;
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

fn setup() -> (TurtleInterpreter, SymbolTable) {
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

    // Intern standard symbols + PBR symbols
    interner.intern("F").unwrap();
    interner.intern("'").unwrap(); // Color
    interner.intern(",").unwrap(); // Material
    interner.intern("@").unwrap(); // Metallic
    interner.intern("#").unwrap(); // Roughness

    interpreter.populate_standard_symbols(&interner);
    (interpreter, interner)
}

#[test]
fn test_pbr_state_changes() {
    let (interpreter, interner) = setup();

    let f_id = interner.resolve_id("F").unwrap();
    let color_id = interner.resolve_id("'").unwrap();
    let mat_id = interner.resolve_id(",").unwrap();
    let metal_id = interner.resolve_id("@").unwrap();
    let rough_id = interner.resolve_id("#").unwrap();

    let mut state = SymbiosState::new();

    // Sequence:
    // 1. Set Color Red (1,0,0)
    // 2. Set Material 1
    // 3. Set Metallic 0.8
    // 4. Set Roughness 0.2
    // 5. Draw F(1)

    state.push(color_id, 0.0, &[1.0, 0.0, 0.0]).unwrap();
    state.push(mat_id, 0.0, &[1.0]).unwrap();
    state.push(metal_id, 0.0, &[0.8]).unwrap();
    state.push(rough_id, 0.0, &[0.2]).unwrap();
    state.push(f_id, 0.0, &[1.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    // Check results on the generated node
    // Note: F generates Start and End nodes. Both should inherit state.

    let strand = &skeleton.strands[0];
    assert_eq!(strand.len(), 2);

    let point = strand[1]; // Check the end point

    // Color (RGBA)
    assert_relative_eq!(point.color.x, 1.0);
    assert_relative_eq!(point.color.y, 0.0);
    assert_relative_eq!(point.color.z, 0.0);
    assert_relative_eq!(point.color.w, 1.0); // Default Alpha

    // Material
    assert_eq!(point.material_id, 1);

    // PBR
    assert_relative_eq!(point.metallic, 0.8);
    assert_relative_eq!(point.roughness, 0.2);
}
