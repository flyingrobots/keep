//! This module owns repository descriptor-allocation policy tests.

use super::CHILD_DESCRIPTOR_MINIMUM;

#[test]
fn child_directory_descriptors_stay_above_standard_streams() {
    assert_eq!(CHILD_DESCRIPTOR_MINIMUM, 3);
}
