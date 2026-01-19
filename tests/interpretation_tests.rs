use approx::assert_relative_eq;
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

fn setup_interpreter() -> (TurtleInterpreter, SymbolTable) {
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

    // Intern standard symbols to get IDs
    interner.intern("F").unwrap();
    interner.intern("+").unwrap();
    interner.intern("[").unwrap();
    interner.intern("]").unwrap();

    interpreter.populate_standard_symbols(&interner);
    (interpreter, interner)
}

#[test]
fn test_draw_forward() {
    let (interpreter, interner) = setup_interpreter();
    let f_id = interner.resolve_id("F").unwrap();

    let mut state = SymbiosState::new();
    // F(10)
    state.push(f_id, 0.0, &[10.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    // Expect 2 points: Start (0,0,0) and End (0,10,0) [Up is Y]
    assert_eq!(skeleton.strands.len(), 1);
    assert_eq!(skeleton.strands[0].len(), 2);

    let start = skeleton.strands[0][0].position;
    let end = skeleton.strands[0][1].position;

    assert_relative_eq!(start.x, 0.0);
    assert_relative_eq!(start.y, 0.0);
    assert_relative_eq!(start.z, 0.0);

    assert_relative_eq!(end.x, 0.0);
    assert_relative_eq!(end.y, 10.0);
    assert_relative_eq!(end.z, 0.0);
}

#[test]
fn test_rotation_yaw() {
    let (interpreter, interner) = setup_interpreter();
    let f_id = interner.resolve_id("F").unwrap();
    let plus_id = interner.resolve_id("+").unwrap();

    let mut state = SymbiosState::new();
    // F(10) +(90) F(10)
    // Up, Turn 90 Z, Up (which is now Left in local space)
    state.push(f_id, 0.0, &[10.0]).unwrap();
    state.push(plus_id, 0.0, &[90.0]).unwrap();
    state.push(f_id, 0.0, &[10.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    let p1 = skeleton.strands[0][1].position;
    let p2 = skeleton.strands[0][2].position;

    // Segment 1: (0,0,0) -> (0,10,0)
    assert_relative_eq!(p1.y, 10.0);

    // Segment 2: After 90 deg rotation around Z
    // Local Y becomes global -X (Right-handed coords? verification needed)
    // Let's check magnitude/direction rather than strict axis first
    let seg2 = p2 - p1;
    assert_relative_eq!(seg2.length(), 10.0);
    assert_relative_eq!(seg2.y, 0.0, epsilon = 0.001);
    assert!(seg2.x.abs() > 9.0); // Should be horizontal
}

#[test]
fn test_branching_topology() {
    let (interpreter, interner) = setup_interpreter();
    let f_id = interner.resolve_id("F").unwrap();
    let push_id = interner.resolve_id("[").unwrap();
    let pop_id = interner.resolve_id("]").unwrap();

    let mut state = SymbiosState::new();
    // F(10) [ F(5) ] F(10)
    state.push(f_id, 0.0, &[10.0]).unwrap();
    state.push(push_id, 0.0, &[]).unwrap();
    state.push(f_id, 0.0, &[5.0]).unwrap();
    state.push(pop_id, 0.0, &[]).unwrap();
    state.push(f_id, 0.0, &[10.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    // Strands logic:
    // Strand 0: F(10) -> (0,0,0) to (0,10,0)
    // Push -> New Strand 1: F(5) -> (0,10,0) to (0,15,0)
    // Pop -> New Strand 2: F(10) -> (0,10,0) to (0,20,0)

    // We expect 3 distinct visual strands because Pop breaks continuity
    assert_eq!(skeleton.strands.len(), 3);

    let root_end = skeleton.strands[0].last().unwrap().position;
    let branch_start = skeleton.strands[1].first().unwrap().position;
    let trunk_resume_start = skeleton.strands[2].first().unwrap().position;

    assert_relative_eq!(root_end.y, 10.0);
    assert_relative_eq!(branch_start.y, 10.0); // Branch starts where root ended
    assert_relative_eq!(trunk_resume_start.y, 10.0); // Trunk resumes where root ended

    let branch_end = skeleton.strands[1].last().unwrap().position;
    assert_relative_eq!(branch_end.y, 15.0);

    let trunk_end = skeleton.strands[2].last().unwrap().position;
    assert_relative_eq!(trunk_end.y, 20.0);
}
