use approx::assert_relative_eq;
use glam::Vec3;
use symbios::{SymbiosState, SymbolTable};
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

fn setup() -> (TurtleInterpreter, SymbolTable) {
    let mut interner = SymbolTable::new();
    let mut interpreter = TurtleInterpreter::new(TurtleConfig::default());

    // Intern rotation symbols
    interner.intern("F").unwrap(); // Draw
    interner.intern("+").unwrap(); // Yaw +
    interner.intern("-").unwrap(); // Yaw -
    interner.intern("&").unwrap(); // Pitch +
    interner.intern("^").unwrap(); // Pitch -
    interner.intern("\\").unwrap(); // Roll +
    interner.intern("/").unwrap(); // Roll -
    interner.intern("|").unwrap(); // Turn Around
    interner.intern("$").unwrap(); // Vertical Align

    interpreter.populate_standard_symbols(&interner);
    (interpreter, interner)
}

/// Helper to run a simple string sequence and get the final position
fn run_sequence(
    sequence: &str,
    interpreter: &TurtleInterpreter,
    interner: &mut SymbolTable,
) -> Vec3 {
    let mut state = SymbiosState::new();

    // Simple parser for test strings like "F(10) +(90) F(10)"
    for token in sequence.split_whitespace() {
        if let Some(start_paren) = token.find('(') {
            let sym = &token[..start_paren];
            let val_str = &token[start_paren + 1..token.len() - 1];
            let val: f32 = val_str.parse().unwrap();

            let id = interner
                .get_or_intern(sym)
                .unwrap_or_else(|_| interner.intern(sym).unwrap());
            state.push(id, 0.0, &[val.into()]).unwrap();
        } else {
            let id = interner
                .get_or_intern(token)
                .unwrap_or_else(|_| interner.intern(token).unwrap());
            state.push(id, 0.0, &[]).unwrap();
        }
    }

    let skeleton = interpreter.build_skeleton(&state);
    skeleton
        .strands
        .last()
        .and_then(|s| s.last())
        .unwrap()
        .position
}

#[test]
fn test_yaw_rotations() {
    let (interpreter, mut interner) = setup();

    // Default Up is +Y.
    // Yaw rotates around local Z.

    // 1. Left Turn (+): +Y -> -X
    // F(10) +(90) F(10) -> (0, 10, 0) + (-10, 0, 0) = (-10, 10, 0)
    let pos_left = run_sequence("F(10) +(90) F(10)", &interpreter, &mut interner);
    assert_relative_eq!(pos_left.x, -10.0, epsilon = 0.001);
    assert_relative_eq!(pos_left.y, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_left.z, 0.0, epsilon = 0.001);

    // 2. Right Turn (-): +Y -> +X
    // F(10) -(90) F(10) -> (0, 10, 0) + (10, 0, 0) = (10, 10, 0)
    let pos_right = run_sequence("F(10) -(90) F(10)", &interpreter, &mut interner);
    assert_relative_eq!(pos_right.x, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_right.y, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_right.z, 0.0, epsilon = 0.001);
}

#[test]
fn test_pitch_rotations() {
    let (interpreter, mut interner) = setup();

    // Pitch rotates around local X.
    // Initial: Up=+Y, Forward=+Z, Right=+X

    // 1. Pitch Down (&): +Y -> +Z
    // F(10) &(90) F(10) -> (0, 10, 0) + (0, 0, 10) = (0, 10, 10)
    let pos_down = run_sequence("F(10) &(90) F(10)", &interpreter, &mut interner);
    assert_relative_eq!(pos_down.x, 0.0, epsilon = 0.001);
    assert_relative_eq!(pos_down.y, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_down.z, 10.0, epsilon = 0.001);

    // 2. Pitch Up (^): +Y -> -Z
    // F(10) ^(90) F(10) -> (0, 10, 0) + (0, 0, -10) = (0, 10, -10)
    let pos_up = run_sequence("F(10) ^(90) F(10)", &interpreter, &mut interner);
    assert_relative_eq!(pos_up.x, 0.0, epsilon = 0.001);
    assert_relative_eq!(pos_up.y, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_up.z, -10.0, epsilon = 0.001);
}

#[test]
fn test_roll_rotations_compound() {
    let (interpreter, mut interner) = setup();

    // Roll rotates around local Y (the movement axis).
    // Rolling alone does not change the next F position.
    // We must Roll THEN Pitch/Yaw to see the effect.

    // Control: Pitch 90 -> Moves in +Z
    let pos_control = run_sequence("&(90) F(10)", &interpreter, &mut interner);
    assert_relative_eq!(pos_control.z, 10.0, epsilon = 0.001);

    // Experiment: Roll 90, Then Pitch 90
    // Initial: Right=+X, Up=+Y, Fwd=+Z
    // \(90): Rotates around Y. Right(+X) becomes Back(-Z)? Or Forward(+Z)?
    // Bevy/Glam uses Right-Handed coordinates.
    // If we Roll 90 around Y: X axis rotates to -Z.
    // Then Pitch (&90) rotates around the NEW local X (which is World -Z).
    // Rotating around -Z moves Y to -X.
    // So F(10) should move to (-10, 0, 0).

    let pos_rolled = run_sequence("/(90) &(90) F(10)", &interpreter, &mut interner);

    // Check Result
    assert_relative_eq!(pos_rolled.x, -10.0, epsilon = 0.001);
    assert_relative_eq!(pos_rolled.y, 0.0, epsilon = 0.001);
    assert_relative_eq!(pos_rolled.z, 0.0, epsilon = 0.001);


    let pos_rolled = run_sequence("\\(90) &(90) F(10)", &interpreter, &mut interner);

    // Check Result
    assert_relative_eq!(pos_rolled.x, 10.0, epsilon = 0.001);
    assert_relative_eq!(pos_rolled.y, 0.0, epsilon = 0.001);
    assert_relative_eq!(pos_rolled.z, 0.0, epsilon = 0.001);
}

#[test]
fn test_turn_around() {
    let (interpreter, mut interner) = setup();

    // | rotates 180 around local Z (Yaw)
    // Up (+Y) becomes Down (-Y).

    let pos = run_sequence("F(10) | F(5)", &interpreter, &mut interner);
    // (0, 10, 0) + (0, -5, 0) = (0, 5, 0)

    assert_relative_eq!(pos.y, 5.0, epsilon = 0.001);
    assert_relative_eq!(pos.x, 0.0, epsilon = 0.001);
    assert_relative_eq!(pos.z, 0.0, epsilon = 0.001);
}

#[test]
fn test_vertical_align() {
    let (interpreter, mut interner) = setup();

    // $ rotates the turtle to align its Up vector with World Y?
    // Actually, in standard L-systems, $ rotates the turtle around its Heading (Up)
    // so that the "Left" vector is horizontal (perpendicular to World Up).
    // It fixes the "Roll" to a canonical orientation.
    // However, the current implementation in `interpreter.rs` does:
    // let h = turtle.up(); let v = Vec3::Y; ... Quat::from_mat3
    // This implementation actually realigns the turtle's rotation so that
    // its UP vector is H, but its ROLL is minimized relative to World Y.

    // Setup: Pitch 45 deg, then Roll 45 deg. The frame is now twisted.
    // Then applying $ should untwist it relative to the horizon.

    // Without $:
    // &(45) -> Tilted forward.
    // \(90) -> Rolled 90 degrees.
    // +(90) -> Yaw (around tilted local Z).

    // With $:
    // &(45) -> Tilted forward.
    // \(90) -> Rolled.
    // $ -> Should reset the Roll component so Left/Right are horizontal.
    // Then a Yaw +(90) should act purely horizontally?
    // No, +(90) is local Z.

    // Let's test stability:
    // 1. Move Up-Right diagonal
    // 2. Apply $
    // 3. Pitch (&) -> Should move purely in the vertical plane defined by current heading

    // Path 1: Pitch 45, Roll 90. Local X is now pointing Up-ish?
    // If we Pitch again, we move sideways.
    let pos_twisted = run_sequence("&(45) \\(90) &(90) F(10)", &interpreter, &mut interner);

    // Path 2: Pitch 45, Roll 90, Align ($), Pitch 90.
    // Align should reset the roll.
    // Local X should become horizontal again.
    // Pitching around horizontal X means moving purely in Y/Z plane (no X change).
    let pos_aligned = run_sequence("&(45) \\(90) $ &(90) F(10)", &interpreter, &mut interner);

    // In the twisted version, X should be non-zero because we pitched around a rolled axis.
    assert!(pos_twisted.x.abs() > 0.1, "Twisted turtle should move in X");

    // In the aligned version, we reset the frame so Local X is horizontal.
    // Pitching around a horizontal axis means we stay in the Y/Z plane (assuming we started there).
    // Since we started moving Vertical (Y), Pitched 45 (now Y/Z), Aligned (still Y/Z heading),
    // Pitching again should keep us in Y/Z.
    assert_relative_eq!(pos_aligned.x, 0.0, epsilon = 0.001);
}
