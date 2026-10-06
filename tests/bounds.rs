use gnw_render_probe::bounds::*;

fn request(offset: u32, count: u32) -> Request {
    Request {
        control_fingerprint: CONTROL_FINGERPRINT,
        storage_addresses: [0x10000000, 0x20000000, 0x30000000],
        storage_sizes: [STOCK_VERTEX_BYTES, STOCK_SKIN_BYTES, STOCK_VERTEX_BYTES],
        uniform_sizes: [80, 5776],
        groups: [count.div_ceil(256) as i32, 1, 1],
        constants: [offset, 28, offset / 28 * 24, 24, count, 2, 0, 1, 1, 1],
    }
}

#[test]
fn every_stock_object_preserves_every_written_record() {
    assert_eq!(OBJECTS.len(), 40);
    for &(offset, count) in OBJECTS {
        let r = request(offset, count);
        let end = bounded_output(&r).unwrap();
        assert_eq!(end % 256, 0);
        assert!(end >= u64::from(offset) + 28 * u64::from(count));
        assert!(end - (u64::from(offset) + 28 * u64::from(count)) < 256);
        assert!(end <= STOCK_VERTEX_BYTES);
    }
}

#[test]
fn idle_hand_and_largest_stock_object_have_measured_bounds() {
    assert_eq!(bounded_output(&request(301448, 405)), Some(312832));
    let max = OBJECTS
        .iter()
        .map(|&(o, c)| bounded_output(&request(o, c)).unwrap())
        .max();
    assert_eq!(max, Some(740096));
}

#[test]
fn mismatched_program_layout_or_dispatch_falls_back() {
    let good = request(301448, 405);
    let mut r = good.clone();
    r.control_fingerprint ^= 1;
    assert_eq!(bounded_output(&r), None);
    for slot in 0..3 {
        let mut r = good.clone();
        r.storage_sizes[slot] -= 1;
        assert_eq!(bounded_output(&r), None);
        let mut r = good.clone();
        r.storage_addresses[slot] = 0;
        assert_eq!(bounded_output(&r), None);
    }
    let mut r = good.clone();
    r.storage_addresses[2] = r.storage_addresses[0];
    assert_eq!(bounded_output(&r), None);
    for i in 0..10 {
        let mut r = good.clone();
        r.constants[i] ^= 1;
        assert_eq!(bounded_output(&r), None);
    }
    for i in 0..3 {
        let mut r = good.clone();
        r.groups[i] += 1;
        assert_eq!(bounded_output(&r), None);
    }
    for i in 0..2 {
        let mut r = good.clone();
        r.uniform_sizes[i] -= 1;
        assert_eq!(bounded_output(&r), None);
    }
}

#[test]
fn malformed_counts_offsets_and_sizes_never_wrap_into_an_accepted_range() {
    for value in [0, 1, u32::MAX, u32::MAX - 255] {
        let mut r = request(301448, 405);
        r.constants[4] = value;
        assert_eq!(bounded_output(&r), None);
        let mut r = request(301448, 405);
        r.constants[0] = value;
        assert_eq!(bounded_output(&r), None);
    }
    let mut r = request(301448, 405);
    r.storage_sizes[2] = u64::MAX;
    assert_eq!(bounded_output(&r), None);
}
