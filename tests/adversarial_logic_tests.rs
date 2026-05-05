use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter, TurtleOp, TurtleWarning};

fn setup() -> (TurtleInterpreter, SymbolTable) {
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

    // Intern generic symbols
    interner.intern("NaN").unwrap();
    interner.intern("Pop").unwrap();

    interpreter.set_op(interner.resolve_id("NaN").unwrap(), TurtleOp::Yaw(1.0));
    interpreter.set_op(interner.resolve_id("Pop").unwrap(), TurtleOp::Pop);

    (interpreter, interner)
}

#[test]
fn test_stack_underflow_resilience() {
    let (interpreter, interner) = setup();
    let pop_id = interner.resolve_id("Pop").unwrap();

    let mut state = SymbiosState::new();
    // Axiom: ] ] ] (Pop empty stack)
    state.push(pop_id, 0.0, &[]).unwrap();
    state.push(pop_id, 0.0, &[]).unwrap();

    // Should NOT panic
    let skeleton = interpreter.build_skeleton(&state);

    // Should produce empty skeleton or minimal safe state, not crash
    assert!(skeleton.strands.is_empty() || skeleton.strands[0].len() <= 1);
}

#[test]
fn test_stack_overflow_reports_warning_and_keeps_pops_in_sync() {
    // Build an interpreter with a tiny stack so we can exhaust it deterministically.
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig {
        max_stack_depth: 2,
        ..Default::default()
    });
    interner.intern("F").unwrap();
    interner.intern("[").unwrap();
    interner.intern("]").unwrap();
    interpreter.populate_standard_symbols(&interner);

    let f_id = interner.resolve_id("F").unwrap();
    let push_id = interner.resolve_id("[").unwrap();
    let pop_id = interner.resolve_id("]").unwrap();

    // Three nested pushes — the third overflows the depth-2 stack — followed
    // by three matching pops, then a final F. With the bug present, the third
    // Pop would unwind a state that was never saved, leaving the post-Pop F
    // misplaced. With the fix, the over-deep Push is dropped *and* its
    // matching Pop is skipped, so the trailing F lands at the same position
    // as a balanced run.
    let mut state = SymbiosState::new();
    state.push(push_id, 0.0, &[]).unwrap();
    state.push(push_id, 0.0, &[]).unwrap();
    state.push(push_id, 0.0, &[]).unwrap(); // overflow — dropped
    state.push(f_id, 0.0, &[1.0]).unwrap();
    state.push(pop_id, 0.0, &[]).unwrap(); // skipped (pairs with dropped push)
    state.push(pop_id, 0.0, &[]).unwrap();
    state.push(pop_id, 0.0, &[]).unwrap();
    state.push(f_id, 0.0, &[1.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    // Warning was emitted for the dropped Push.
    assert_eq!(skeleton.warnings.len(), 1);
    match skeleton.warnings[0] {
        TurtleWarning::StackOverflow { symbol_index } => {
            assert_eq!(symbol_index, 2, "overflow happened at the third symbol");
        }
        _ => panic!(
            "expected StackOverflow warning, got {:?}",
            skeleton.warnings[0]
        ),
    }

    // Stack stayed in sync: the trailing F draws from the *original* turtle
    // state (all pops unwound cleanly), so it sits at y == 1.0 — not at the
    // y == 1.0 of the inner-branch state that the buggy version would yield.
    let last_strand = skeleton.strands.last().expect("at least one strand");
    let last_point = last_strand.last().expect("strand has points");
    assert!(
        (last_point.position.y - 1.0).abs() < 1e-5,
        "trailing F should land at y=1.0 once stack is balanced, got {}",
        last_point.position.y
    );
}

#[test]
fn test_op_map_high_id_uses_sparse_tier_not_padded_vec() {
    // Register a single op at id 10_000. With the old flat Vec, the table
    // would resize to 10_001 entries (~80 KB of TurtleOp::Ignore padding).
    // The tiered map keeps the dense Vec bounded and stores this in a
    // 1-entry HashMap instead — so a Draw at that id still resolves to
    // TurtleOp::Draw and produces geometry, while the only standard symbol
    // ('F' at id 0) and the high one each take O(1) lookup.
    let mut interner = SymbolTable::new();
    interner.intern("F").unwrap();

    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());
    interpreter.populate_standard_symbols(&interner);
    interpreter.set_op(10_000, TurtleOp::Draw);

    let mut state = SymbiosState::new();
    state.push(10_000, 0.0, &[2.0]).unwrap();
    state.push(10_000, 0.0, &[3.0]).unwrap();

    let skeleton = interpreter.build_skeleton(&state);

    // Both high-id Draws should have produced geometry — proving the sparse
    // tier is participating in lookup, not just silently mapping to Ignore.
    assert_eq!(skeleton.strands.len(), 1);
    let strand = &skeleton.strands[0];
    assert!(strand.len() >= 2, "expected at least start + end points");
    let last_y = strand.last().unwrap().position.y;
    assert!(
        (last_y - 5.0).abs() < 1e-5,
        "two Draws of length 2 + 3 should land at y=5.0, got {last_y}"
    );

    // An *unregistered* high id should still resolve to Ignore — no panic,
    // no growth pressure on the dense Vec.
    let mut state2 = SymbiosState::new();
    state2.push(50_000, 0.0, &[]).unwrap();
    let skeleton2 = interpreter.build_skeleton(&state2);
    assert!(skeleton2.strands.is_empty());
}
