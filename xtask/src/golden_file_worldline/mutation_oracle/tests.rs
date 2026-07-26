use super::{GoldenError, apply_fixed_width};

#[test]
fn fixed_width_mutations_require_the_declared_value_width() {
    for value in ["", "02", "000203"] {
        let mut changed = [0_u8; 2];
        let result = apply_fixed_width(&mut changed, "set-u16-be", 0, value, "set-version");
        assert!(matches!(
            result,
            Err(GoldenError::Violation(ref message))
                if message == "set-version: mutation value must be exactly 2 bytes"
        ));
    }
}

#[test]
fn fixed_width_mutations_admit_the_declared_value_width() {
    let mut changed = [0_u8; 2];
    let result = apply_fixed_width(&mut changed, "set-u16-be", 0, "0002", "set-version");
    assert!(result.is_ok());
    assert_eq!(changed, [0_u8, 2]);
}
