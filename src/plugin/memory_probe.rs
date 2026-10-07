use super::*;

type InitPool = unsafe extern "C" fn(Object, *const c_void) -> u8;
type FinishPool = unsafe extern "C" fn(Object);
type PoolAddress = unsafe extern "C" fn(*const c_void) -> u64;
type BuilderMemory = unsafe extern "C" fn(*const c_void) -> *const c_void;
type BuilderSize = unsafe extern "C" fn(*const c_void) -> usize;
type SetShaders = unsafe extern "C" fn(Object, i32, *const ShaderData) -> u8;

#[repr(C)]
struct ShaderData {
    data: u64,
    control: *const c_void,
}
#[derive(Clone, Copy)]
struct Pool {
    handle: usize,
    cpu: usize,
    gpu: u64,
    size: usize,
}
#[derive(Clone, Copy)]
struct Program {
    handle: usize,
    shader_data: u64,
    control_hash: u64,
    code_hash: u64,
    epoch: usize,
}
#[repr(C)]
#[derive(Default)]
struct MemoryInfo {
    address: u64,
    size: u64,
    kind: u32,
    attributes: u32,
    permissions: u32,
    device_refs: u32,
    ipc_refs: u32,
    padding: u32,
}

static POOLS: Mutex<Vec<Pool>> = Mutex::new(Vec::new());
static PROGRAMS: Mutex<Vec<Program>> = Mutex::new(Vec::new());
static POOL_INIT: AtomicUsize = AtomicUsize::new(0);
static POOL_FINISH: AtomicUsize = AtomicUsize::new(0);
static POOL_ADDRESS: AtomicUsize = AtomicUsize::new(0);
static BUILDER_MEMORY: AtomicUsize = AtomicUsize::new(0);
static BUILDER_SIZE: AtomicUsize = AtomicUsize::new(0);
static POOL_EPOCH: AtomicUsize = AtomicUsize::new(1);
static PROGRAM_FINISH: AtomicUsize = AtomicUsize::new(0);
static SET_SHADERS: AtomicUsize = AtomicUsize::new(0);

// Read-only probe. Validate a mapped readable span before touching opaque memory.
// This does not establish object identity or ownership; it prevents a bad read.
fn readable(address: usize, size: usize) -> bool {
    let Some(end) = address.checked_add(size) else {
        return false;
    };
    if address == 0 {
        return false;
    }
    let mut info = MemoryInfo::default();
    let result: usize;
    // SVC 0x6 follows the caller-saved register contract of libnx svcQueryMemory.
    // Declare those clobbers so the optimizer preserves live pointers across the
    // trap. x0/x1 return results; x2 is consumed and explicitly discarded.
    unsafe {
        std::arch::asm!(
            "svc #0x6",
            inlateout("x0") &mut info as *mut MemoryInfo => result,
            lateout("x1") _,
            inlateout("x2") address => _,
            clobber_abi("C"),
            options(nostack)
        );
    }
    result == 0
        && info.permissions & 1 != 0
        && address as u64 >= info.address
        && info
            .address
            .checked_add(info.size)
            .is_some_and(|limit| end as u64 <= limit)
}

fn control_hash(control: *const c_void) -> u64 {
    if !readable(control as usize, 64) {
        return 0;
    }
    let mut hash = 0xcbf29ce484222325u64;
    for i in 0..64 {
        let byte = unsafe { control.cast::<u8>().add(i).read_volatile() };
        hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
    }
    hash
}

unsafe extern "C" fn initialize(pool: Object, builder: *const c_void) -> u8 {
    let original: InitPool = std::mem::transmute(POOL_INIT.load(Ordering::Acquire));
    let result = original(pool, builder);
    let gpu_fn = POOL_ADDRESS.load(Ordering::Acquire);
    let cpu_fn = BUILDER_MEMORY.load(Ordering::Acquire);
    let size_fn = BUILDER_SIZE.load(Ordering::Acquire);
    if result != 0 && gpu_fn != 0 && cpu_fn != 0 && size_fn != 0 {
        let gpu_fn: PoolAddress = std::mem::transmute(gpu_fn);
        let cpu_fn: BuilderMemory = std::mem::transmute(cpu_fn);
        let size_fn: BuilderSize = std::mem::transmute(size_fn);
        let entry = Pool {
            handle: pool as usize,
            cpu: cpu_fn(builder) as usize,
            gpu: gpu_fn(pool),
            size: size_fn(builder),
        };
        if let Ok(mut pools) = POOLS.lock() {
            pools.retain(|p| p.handle != entry.handle);
            POOL_EPOCH.fetch_add(1, Ordering::AcqRel);
            if entry.cpu != 0 && entry.gpu != 0 && entry.size != 0 && pools.len() < 4096 {
                pools.push(entry);
            }
        }
    }
    result
}

unsafe extern "C" fn finalize(pool: Object) {
    if let Ok(mut pools) = POOLS.lock() {
        pools.retain(|p| p.handle != pool as usize);
        POOL_EPOCH.fetch_add(1, Ordering::AcqRel);
    }
    let original: FinishPool = std::mem::transmute(POOL_FINISH.load(Ordering::Acquire));
    original(pool);
}

unsafe extern "C" fn shaders(program: Object, count: i32, data: *const ShaderData) -> u8 {
    let original: SetShaders = std::mem::transmute(SET_SHADERS.load(Ordering::Acquire));
    let result = original(program, count, data);
    // The candidate pre-skin program has one shader; multistage programs stay unclassified.
    if let Ok(mut programs) = PROGRAMS.lock() {
        programs.retain(|p| p.handle != program as usize);
        if result != 0
            && count == 1
            && readable(data as usize, std::mem::size_of::<ShaderData>())
            && programs.len() < 4096
        {
            let d = &*data;
            programs.push(Program {
                handle: program as usize,
                shader_data: d.data,
                control_hash: control_hash(d.control),
                code_hash: 0,
                epoch: 0,
            });
        }
    }
    result
}

unsafe extern "C" fn finish_program(program: Object) {
    if let Ok(mut programs) = PROGRAMS.lock() {
        programs.retain(|p| p.handle != program as usize);
    }
    let original: FinishPool = std::mem::transmute(PROGRAM_FINISH.load(Ordering::Acquire));
    original(program);
}

// The local Eden cache entry keyed c478aab12efe386f contains this entire
// 4,600-byte compute program at guest shader offset 0x16300 (256,1,1).
// Only its fingerprint is embedded; no game shader bytes are distributed.
const CODE_BYTES: usize = 4600;
const CODE_HASH: u64 = 0x3869_d88a_d69e_ee6a;

fn mapped<T>(range: Range, read: impl FnOnce(usize) -> Option<T>) -> Option<T> {
    let pools = POOLS.lock().ok()?;
    let mut found = None;
    for p in pools.iter() {
        let Some(offset) = range.address.checked_sub(p.gpu) else {
            continue;
        };
        let Some(end) = offset.checked_add(range.size) else {
            continue;
        };
        if end > p.size as u64 {
            continue;
        }
        let cpu = p.cpu.checked_add(usize::try_from(offset).ok()?)?;
        if found.replace(cpu).is_some() {
            return None;
        }
    }
    let cpu = found?;
    readable(cpu, usize::try_from(range.size).ok()?).then_some(())?;
    read(cpu)
}

pub(super) fn verified_constants(program: usize, uniform: Range) -> Option<[u32; 10]> {
    if PROGRAM_FINISH.load(Ordering::Acquire) == 0 || uniform.size != 80 {
        return None;
    }
    let p = *PROGRAMS.lock().ok()?.iter().find(|p| p.handle == program)?;
    if p.control_hash != crate::bounds::CONTROL_FINGERPRINT {
        return None;
    }
    let epoch = POOL_EPOCH.load(Ordering::Acquire);
    let hash = if p.epoch == epoch {
        p.code_hash
    } else {
        let hash = mapped(
            Range {
                address: p.shader_data,
                size: CODE_BYTES as u64,
            },
            |cpu| {
                let mut hash = 0xcbf29ce484222325u64;
                for i in 0..CODE_BYTES {
                    let byte = unsafe { (cpu as *const u8).add(i).read_volatile() };
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                }
                Some(hash)
            },
        )?;
        if POOL_EPOCH.load(Ordering::Acquire) != epoch {
            return None;
        }
        if let Ok(mut programs) = PROGRAMS.lock() {
            if let Some(current) = programs
                .iter_mut()
                .find(|v| v.handle == program && v.shader_data == p.shader_data)
            {
                current.code_hash = hash;
                current.epoch = epoch;
            }
        }
        log(format_args!(
            "shader code bytes={CODE_BYTES} fnv={hash:#018x} verified={}",
            hash == CODE_HASH
        ));
        hash
    };
    if hash != CODE_HASH {
        return None;
    }
    mapped(uniform, |cpu| {
        if cpu & 3 != 0 {
            return None;
        }
        let mut words = [0u32; 10];
        for (i, word) in words.iter_mut().enumerate() {
            *word = unsafe { (cpu as *const u32).add(i).read_volatile() };
        }
        Some(words)
    })
}

pub(super) fn describe_dispatch(program: usize, uniform: Range) {
    let verified = verified_constants(program, uniform).is_some();
    log(format_args!(
        "full shader + mapped constants verified={verified}"
    ));
    if let Ok(programs) = PROGRAMS.lock() {
        if let Some(p) = programs.iter().find(|p| p.handle == program) {
            log(format_args!(
                "identity shader_gpu={:#x} control64_fnv={:#018x} (observed, not allowlisted)",
                p.shader_data, p.control_hash
            ));
        } else {
            log(format_args!("identity UNOBSERVED program={program:#x}"));
        }
    }
    if uniform.size != 80 {
        return;
    }
    if let Ok(pools) = POOLS.lock() {
        let found = pools
            .iter()
            .filter_map(|p| {
                let offset = uniform.address.checked_sub(p.gpu)?;
                let end = offset.checked_add(uniform.size)?;
                if end > p.size as u64 {
                    return None;
                }
                Some((p, p.cpu.checked_add(offset as usize)?))
            })
            .collect::<Vec<_>>();
        if found.len() != 1 {
            log(format_args!(
                "ubo0 mapping matches={} pools={}",
                found.len(),
                pools.len()
            ));
            return;
        }
        let (pool, cpu) = found[0];
        if cpu & 3 != 0 || !readable(cpu, 80) {
            log(format_args!("ubo0 CPU span unreadable"));
            return;
        }
        let mut words = [0u32; 20];
        for (i, word) in words.iter_mut().enumerate() {
            *word = unsafe { (cpu as *const u32).add(i).read_volatile() };
        }
        log(format_args!(
            "ubo0 pool={:#x} cpu={cpu:#x} words={words:?}",
            pool.handle
        ));
    }
}

pub(super) fn resolve(name: &[u8], result: *const c_void) -> Option<*const c_void> {
    let (hook, slot): (*const c_void, &AtomicUsize) = match name {
        b"nvnMemoryPoolInitialize" => (initialize as *const c_void, &POOL_INIT),
        b"nvnMemoryPoolFinalize" => (finalize as *const c_void, &POOL_FINISH),
        b"nvnProgramFinalize" => (finish_program as *const c_void, &PROGRAM_FINISH),
        b"nvnProgramSetShaders" => (shaders as *const c_void, &SET_SHADERS),
        b"nvnMemoryPoolGetBufferAddress" => (std::ptr::null(), &POOL_ADDRESS),
        b"nvnMemoryPoolBuilderGetMemory" => (std::ptr::null(), &BUILDER_MEMORY),
        b"nvnMemoryPoolBuilderGetSize" => (std::ptr::null(), &BUILDER_SIZE),
        _ => return None,
    };
    if hook.is_null() {
        if !result.is_null() && slot.swap(result as usize, Ordering::AcqRel) == 0 {
            log(format_args!(
                "observed getter {} original={result:p}",
                String::from_utf8_lossy(name)
            ));
        }
        Some(result)
    } else {
        Some(wrap(name, result, hook, slot))
    }
}
