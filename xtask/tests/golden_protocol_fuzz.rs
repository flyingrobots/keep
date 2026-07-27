//! Regression laws for the bounded production-parser fuzz surface.

#![cfg(feature = "golden-protocol-fuzz")]

use xtask::{GoldenProtocolAdmission, admit_golden_protocol};

const VALID_INPUTS: [&[u8]; 9] = [
    b"# keep.golden-file-worldline.identities/v1\n\
      case\tsource_kind\tsource_parameter\trepetitions\tlogical_length\tcanonical_text\tcanonical_binary_hex\n\
      example\tempty-v1\t-\t1\t0\tidentity\t00\n",
    b"# keep.golden-file-worldline.invalid-text/v1\n\
      case\tinput_hex\texpected_outcome\n\
      example\t00\tkeep.identity.malformed_structure\n",
    b"# keep.golden-file-worldline.mutations/v1\n\
      case\ttarget_kind\ttarget_case\toperation\toffset\tvalue_hex\texpected_outcome\n\
      example\tcontent\ttarget\txor-byte\t0\t01\tkeep.content.mismatch\n",
    b"# keep.golden-file-worldline.steps/v1\n\
      step\toperation\tinput_case\tidentity_case\texpected_outcome\n\
      1\tidentify\texample\texample\tkeep.identity.identified\n",
    b"# keep.golden-file-worldline.capabilities/v1\n\
      capability\tposture\tfirst_milestone\towning_issues\tclaim\n\
      keep.example/v1\trequired\tM1\t1\texample claim\n",
    b"canonical-case",
    b"18446744073709551615",
    b"not-an-identity",
    b"00\txor-byte\t0\t01",
];

#[test]
fn every_golden_protocol_parser_is_reachable_with_admitted_input() {
    for (selector, input) in (0_u8..).zip(VALID_INPUTS) {
        assert_eq!(
            admit_golden_protocol(selector, input),
            GoldenProtocolAdmission::Admitted
        );
    }
}

#[test]
fn golden_protocol_fuzz_admission_refuses_unbounded_input() {
    let oversized = vec![b'a'; 1_048_578];
    assert_eq!(
        admit_golden_protocol(0, &oversized),
        GoldenProtocolAdmission::Refused
    );
}
