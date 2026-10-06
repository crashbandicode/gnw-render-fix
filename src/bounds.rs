//! Bounds for one known stock G&W compute layout. No FFI or mutation here.
//! This is an offline experimental allowlist, not online compatibility evidence.

pub const STOCK_VERTEX_BYTES: u64 = 19_518_352;
pub const STOCK_SKIN_BYTES: u64 = 16_730_016;
pub const CONTROL_FINGERPRINT: u64 = 0xbb98_7076_40c6_02a2;

// Stock 13.0.5 c00 mesh, all 40 skinned objects. Source metadata is recorded
// in HANDOFF.md. Each tuple is the original byte offset and vertex count.
pub const OBJECTS: &[(u32, u32)] = &[
    (0, 669),       // LhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (18732, 669),   // LhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (37464, 669),   // LhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (56196, 669),   // LhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (74928, 672),   // RhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (93744, 669),   // RhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (112476, 669),  // RhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (131208, 672),  // RhandMove_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (150024, 676),  // LhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (168952, 676),  // LhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (187880, 676),  // LhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (206808, 676),  // LhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (225736, 676),  // RhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (244664, 676),  // RhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (263592, 676),  // RhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (282520, 676),  // RhandOpen_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (301448, 405),  // RhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (312788, 405),  // RhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (324128, 405),  // RhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (335468, 405),  // RhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (346808, 405),  // LhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (358148, 405),  // LhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (369488, 405),  // LhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (380828, 405),  // LhandHold_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (392168, 405),  // LhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (403508, 405),  // LhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (414848, 405),  // LhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (426188, 405),  // LhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (437528, 405),  // RhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (448868, 405),  // RhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (460208, 405),  // RhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (471548, 405),  // RhandResult_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (482888, 1323), // GW_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (519932, 1307), // GW_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (556528, 1302), // GW_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (592984, 1323), // GW_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
    (664580, 596),  // HeadM_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:0
    (681268, 594),  // HeadM_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:1
    (697900, 998),  // HeadM_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:2
    (725844, 504),  // HeadM_VIS_O_OBJ_O_NOSORT_AS_OPAQUEShape:3
];

#[derive(Clone, Debug)]
pub struct Request {
    pub control_fingerprint: u64,
    pub storage_addresses: [u64; 3],
    pub storage_sizes: [u64; 3],
    pub uniform_sizes: [u64; 2],
    pub groups: [i32; 3],
    pub constants: [u32; 10],
}

/// Return a conservative exclusive byte end, or preserve the original binding.
/// The caller must independently verify the shader and actual memory mapping.
/// A metadata fingerprint is only one guard; it is not a shader-code identity.
pub fn bounded_output(request: &Request) -> Option<u64> {
    let c = request.constants;
    if request.control_fingerprint != CONTROL_FINGERPRINT
        || request.storage_sizes != [STOCK_VERTEX_BYTES, STOCK_SKIN_BYTES, STOCK_VERTEX_BYTES]
        || request.uniform_sizes != [80, 5776]
        || request.storage_addresses.contains(&0)
        || request.storage_addresses[2] == request.storage_addresses[0]
        || request.storage_addresses[2] == request.storage_addresses[1]
        || c[1] != 28
        || c[3] != 24
        || c[5..] != [2, 0, 1, 1, 1]
        || !OBJECTS.contains(&(c[0], c[4]))
        || u64::from(c[2]) != u64::from(c[0]).checked_div(28)?.checked_mul(24)?
        || request.groups != [i32::try_from(c[4].div_ceil(256)).ok()?, 1, 1]
    {
        return None;
    }
    let raw_end = u64::from(c[0]).checked_add(u64::from(c[1]).checked_mul(u64::from(c[4]))?)?;
    let end = raw_end.checked_add(255)? & !255;
    (end != 0 && end <= request.storage_sizes[2]).then_some(end)
}
