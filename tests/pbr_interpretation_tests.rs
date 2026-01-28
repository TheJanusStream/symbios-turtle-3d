use approx::assert_relative_eq;
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

fn setup() -> (TurtleInterpreter, SymbolTable) {
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

    // Intern standard symbols + material symbols
    interner.intern("F").unwrap();
    interner.intern("'").unwrap(); // Color
    interner.intern(",").unwrap(); // Material
    interner.intern(";").unwrap(); // UV Scale
    interner.intern("~").unwrap(); // Spawn

    interpreter.populate_standard_symbols(&interner);
    (interpreter, interner)
}

#[test]
fn test_material_state_changes() {
    let (interpreter, interner) = setup();

    let f_id = interner.resolve_id("F").unwrap();
    let color_id = interner.resolve_id("'").unwrap();
    let mat_id = interner.resolve_id(",").unwrap();
    let uv_id = interner.resolve_id(";").unwrap();

    let mut state = SymbiosState::new();

    // Sequence:
    // 1. Set Color Red (1,0,0)
    // 2. Set Material 1
    // 3. Set UV Scale 2.5
    // 4. Draw F(1)

    state.push(color_id, 0.0, &[1.0, 0.0, 0.0]).unwrap();
    state.push(mat_id, 0.0, &[1.0]).unwrap();
    state.push(uv_id, 0.0, &[2.5]).unwrap();
    state.push(f_id, 0.0, &[1.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    let strand = &skeleton.strands[0];
    assert_eq!(strand.len(), 2);

    let point = strand[1]; // Check the end point

    // Color (RGBA)
    assert_relative_eq!(point.color.x, 1.0);
    assert_relative_eq!(point.color.y, 0.0);
    assert_relative_eq!(point.color.z, 0.0);
    assert_relative_eq!(point.color.w, 1.0); // Default Alpha

    // Material palette ID
    assert_eq!(point.material_id, 1);

    // UV Scale
    assert_relative_eq!(point.uv_scale, 2.5);
}

#[test]
fn test_prop_inherits_turtle_material_state() {
    let (interpreter, interner) = setup();

    let color_id = interner.resolve_id("'").unwrap();
    let mat_id = interner.resolve_id(",").unwrap();
    let spawn_id = interner.resolve_id("~").unwrap();

    let mut state = SymbiosState::new();

    // Set color to green, material to 3, then spawn a prop
    state.push(color_id, 0.0, &[0.0, 1.0, 0.0]).unwrap();
    state.push(mat_id, 0.0, &[3.0]).unwrap();
    state.push(spawn_id, 0.0, &[7.0]).unwrap(); // prop_id=7

    let skeleton = interpreter.build_skeleton(&state);

    assert_eq!(skeleton.props.len(), 1);
    let prop = &skeleton.props[0];

    assert_eq!(prop.prop_id, 7);

    // Color should be green (RGB) with default alpha
    assert_relative_eq!(prop.color.x, 0.0);
    assert_relative_eq!(prop.color.y, 1.0);
    assert_relative_eq!(prop.color.z, 0.0);
    assert_relative_eq!(prop.color.w, 1.0);

    // Material ID
    assert_eq!(prop.material_id, 3);
}

#[test]
fn test_prop_default_material_state() {
    let (interpreter, interner) = setup();

    let spawn_id = interner.resolve_id("~").unwrap();

    let mut state = SymbiosState::new();
    state.push(spawn_id, 0.0, &[0.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    assert_eq!(skeleton.props.len(), 1);
    let prop = &skeleton.props[0];

    // Defaults: white color, material 0
    assert_relative_eq!(prop.color.x, 1.0);
    assert_relative_eq!(prop.color.y, 1.0);
    assert_relative_eq!(prop.color.z, 1.0);
    assert_relative_eq!(prop.color.w, 1.0);
    assert_eq!(prop.material_id, 0);
}

#[test]
fn test_uv_scale_default() {
    let (interpreter, interner) = setup();

    let f_id = interner.resolve_id("F").unwrap();

    let mut state = SymbiosState::new();
    state.push(f_id, 0.0, &[1.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    let point = skeleton.strands[0][0];
    assert_relative_eq!(point.uv_scale, 1.0);
}
