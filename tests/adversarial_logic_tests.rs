use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter, TurtleOp};

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
