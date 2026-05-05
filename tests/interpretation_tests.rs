use approx::assert_relative_eq;
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{STANDARD_TURTLE_SYMBOLS, TurtleConfig, TurtleInterpreter};

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

    // Parent linkage: branch (1) and trunk-resume (2) both fork from the
    // root strand (0).
    assert_eq!(skeleton.strand_parents.len(), skeleton.strands.len());
    assert_eq!(skeleton.strand_parents[0], None, "root has no parent");
    assert_eq!(
        skeleton.strand_parents[1],
        Some(0),
        "branch should point at root"
    );
    assert_eq!(
        skeleton.strand_parents[2],
        Some(0),
        "post-Pop trunk resume should also point at root"
    );
}

#[test]
fn test_strand_parents_nested_branches() {
    // F [ F [ F ] F ] F  — two levels of nesting.
    // Expected strand layout:
    //   0: root (parent None)            — first F
    //   1: outer branch (parent 0)       — opens at first [
    //   2: inner branch (parent 1)       — opens at second [
    //   3: outer-branch resume (parent 1)— closes inner ]
    //   4: trunk resume (parent 0)       — closes outer ]
    let (interpreter, interner) = setup_interpreter();
    let f_id = interner.resolve_id("F").unwrap();
    let push_id = interner.resolve_id("[").unwrap();
    let pop_id = interner.resolve_id("]").unwrap();

    let mut state = SymbiosState::new();
    for (id, params) in [
        (f_id, &[1.0_f64][..]),
        (push_id, &[][..]),
        (f_id, &[1.0][..]),
        (push_id, &[][..]),
        (f_id, &[1.0][..]),
        (pop_id, &[][..]),
        (f_id, &[1.0][..]),
        (pop_id, &[][..]),
        (f_id, &[1.0][..]),
    ] {
        state.push(id, 0.0, params).unwrap();
    }

    let skeleton = interpreter.build_skeleton(&state);
    assert_eq!(skeleton.strand_parents.len(), skeleton.strands.len());
    assert_eq!(skeleton.strand_parents[0], None);
    assert_eq!(skeleton.strand_parents[1], Some(0));
    assert_eq!(skeleton.strand_parents[2], Some(1));
    assert_eq!(skeleton.strand_parents[3], Some(1));
    assert_eq!(skeleton.strand_parents[4], Some(0));
}

#[test]
fn test_with_standard_symbols_matches_manual_setup() {
    // Manual: intern + populate, the boilerplate consumers currently write.
    let mut interner_a = SymbolTable::new();
    for sym in STANDARD_TURTLE_SYMBOLS {
        interner_a.intern(sym).unwrap();
    }
    let mut interpreter_a = TurtleInterpreter::new(TurtleConfig::default());
    interpreter_a.populate_standard_symbols(&interner_a);

    // Convenience: builder helper does both.
    let mut interner_b = SymbolTable::new();
    let interpreter_b =
        TurtleInterpreter::new(TurtleConfig::default()).with_standard_symbols(&mut interner_b);

    // Both interners should resolve every standard symbol to the same ID
    // (insertion order is preserved by the symbol table).
    for sym in STANDARD_TURTLE_SYMBOLS {
        assert_eq!(
            interner_a.resolve_id(sym),
            interner_b.resolve_id(sym),
            "ID drift for symbol {sym:?}"
        );
    }

    // Both interpreters should produce identical skeletons for the same input.
    let f_id = interner_a.resolve_id("F").unwrap();
    let plus_id = interner_a.resolve_id("+").unwrap();
    let mut state = SymbiosState::new();
    state.push(f_id, 0.0, &[3.0]).unwrap();
    state.push(plus_id, 0.0, &[30.0]).unwrap();
    state.push(f_id, 0.0, &[2.0]).unwrap();

    let sk_a = interpreter_a.build_skeleton(&state);
    let sk_b = interpreter_b.build_skeleton(&state);

    assert_eq!(sk_a.strands.len(), sk_b.strands.len());
    for (sa, sb) in sk_a.strands.iter().zip(sk_b.strands.iter()) {
        assert_eq!(sa.len(), sb.len());
        for (pa, pb) in sa.iter().zip(sb.iter()) {
            assert_relative_eq!(pa.position.x, pb.position.x);
            assert_relative_eq!(pa.position.y, pb.position.y);
            assert_relative_eq!(pa.position.z, pb.position.z);
        }
    }
}

#[test]
fn test_with_standard_symbols_idempotent_on_pre_populated_table() {
    // Pre-intern a couple of symbols ourselves; with_standard_symbols should
    // see them and leave their IDs alone, only adding the missing ones.
    let mut interner = SymbolTable::new();
    let pre_f = interner.intern("F").unwrap();
    let pre_plus = interner.intern("+").unwrap();

    let _interpreter =
        TurtleInterpreter::new(TurtleConfig::default()).with_standard_symbols(&mut interner);

    assert_eq!(interner.resolve_id("F"), Some(pre_f));
    assert_eq!(interner.resolve_id("+"), Some(pre_plus));
    // And the helper still added the rest.
    for sym in STANDARD_TURTLE_SYMBOLS {
        assert!(interner.resolve_id(sym).is_some(), "missing symbol {sym:?}");
    }
}
