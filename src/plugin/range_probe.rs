use super::*;

static GETTERS_TRIED: AtomicUsize = AtomicUsize::new(0);
static COMMAND_FREE: AtomicUsize = AtomicUsize::new(0);
static CONTROL_FREE: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "range-probe")]
static RANGE_LOGS: AtomicUsize = AtomicUsize::new(24);

pub(super) unsafe fn resolve_getters(device: *const c_void, resolver: GetProc) {
    if device.is_null() || GETTERS_TRIED.swap(1, Ordering::AcqRel) != 0 {
        return;
    }
    for (name, slot) in [
        (c"nvnCommandBufferGetCommandMemoryFree", &COMMAND_FREE),
        (c"nvnCommandBufferGetControlMemoryFree", &CONTROL_FREE),
    ] {
        let address = resolver(device, name.as_ptr());
        slot.store(address as usize, Ordering::Release);
        log(format_args!(
            "range getter {}={address:p}",
            name.to_string_lossy()
        ));
    }
}

/// Forward the original dispatch exactly once, bracketed by same-address binds.
/// State is a copied observation; no mutex is held across NVN calls.
#[cfg(feature = "range-probe")]
pub(super) unsafe fn dispatch_bounded(
    cmd: Object,
    s: &State,
    groups: [i32; 3],
    dispatch: Dispatch,
) -> bool {
    if RELIABLE.load(Ordering::Acquire) == 0 {
        return false;
    }
    let bind = STORAGE_ONE.load(Ordering::Acquire);
    let command_free = COMMAND_FREE.load(Ordering::Acquire);
    let control_free = CONTROL_FREE.load(Ordering::Acquire);
    if bind == 0 || command_free == 0 || control_free == 0 {
        return false;
    }
    let Some(constants) = memory_probe::verified_constants(s.program, s.uniform[0]) else {
        return false;
    };
    let request = crate::bounds::Request {
        control_fingerprint: crate::bounds::CONTROL_FINGERPRINT,
        storage_addresses: std::array::from_fn(|i| s.storage[i].address),
        storage_sizes: std::array::from_fn(|i| s.storage[i].size),
        uniform_sizes: [s.uniform[0].size, s.uniform[1].size],
        groups,
        constants,
    };
    let Some(end) = crate::bounds::bounded_output(&request) else {
        return false;
    };
    let command_free: unsafe extern "C" fn(*const c_void) -> usize =
        std::mem::transmute(command_free);
    let control_free: unsafe extern "C" fn(*const c_void) -> usize =
        std::mem::transmute(control_free);
    if command_free(cmd) < 512 || control_free(cmd) < 64 {
        return false;
    }
    let bind: BindOne = std::mem::transmute(bind);
    let original = s.storage[2];
    bind(cmd, COMPUTE, 2, original.address, end as usize);
    dispatch(cmd, groups[0], groups[1], groups[2]);
    bind(cmd, COMPUTE, 2, original.address, original.size as usize);
    if take(&RANGE_LOGS) {
        log(format_args!(
            "RANGE slot=2 gpu={:#x} bytes={}->{} base={} count={} restored=true",
            original.address, original.size, end, constants[0], constants[4]
        ));
    }
    true
}
