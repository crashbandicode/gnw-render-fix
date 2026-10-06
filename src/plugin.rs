use std::ffi::{c_char, c_void, CStr};
use std::fmt::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
mod memory_probe;
mod range_probe;
use memory_probe::describe_dispatch;

// Public NVN ABI declarations only. No SDK implementation or gameplay hooks.
const BOOT_NAME: &CStr = c"nvnBootstrapLoader";
const COMPUTE: i32 = 5;
const COMPUTE_BIT: i32 = 32;
const STOCK_GNW_BYTES: u64 = 19_518_352;
type Object = *mut c_void;
type Bootstrap = unsafe extern "C" fn(*const c_char) -> *const c_void;
type GetProc = unsafe extern "C" fn(*const c_void, *const c_char) -> *const c_void;
type BindOne = unsafe extern "C" fn(Object, i32, i32, u64, usize);
type BindMany = unsafe extern "C" fn(Object, i32, i32, i32, *const Range);
type BindProgram = unsafe extern "C" fn(Object, *const c_void, i32);
type Dispatch = unsafe extern "C" fn(Object, i32, i32, i32);
type Begin = unsafe extern "C" fn(Object);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Range {
    address: u64,
    size: u64,
}

#[derive(Clone, Copy, Default)]
struct State {
    cmd: usize,
    program: usize,
    storage: [Range; 16],
    uniform: [Range; 16],
}

static STATES: Mutex<Vec<State>> = Mutex::new(Vec::new());
static LOGS: AtomicUsize = AtomicUsize::new(512);
static DISPATCH_LOGS: AtomicUsize = AtomicUsize::new(64);
static BIND_LOGS: AtomicUsize = AtomicUsize::new(12);
static CAP_LOGS: AtomicUsize = AtomicUsize::new(1);
static END: AtomicUsize = AtomicUsize::new(0);
static FINISH_CMD: AtomicUsize = AtomicUsize::new(0);
static RELIABLE: AtomicUsize = AtomicUsize::new(1);
static CALL: AtomicUsize = AtomicUsize::new(0);
static COPY: AtomicUsize = AtomicUsize::new(0);
static BOOT: AtomicUsize = AtomicUsize::new(0);
static GET_PROC: AtomicUsize = AtomicUsize::new(0);
static STORAGE_ONE: AtomicUsize = AtomicUsize::new(0);
static STORAGE_MANY: AtomicUsize = AtomicUsize::new(0);
static UNIFORM_ONE: AtomicUsize = AtomicUsize::new(0);
static UNIFORM_MANY: AtomicUsize = AtomicUsize::new(0);
static PROGRAM: AtomicUsize = AtomicUsize::new(0);
static DISPATCH: AtomicUsize = AtomicUsize::new(0);
static BEGIN: AtomicUsize = AtomicUsize::new(0);

fn take(counter: &AtomicUsize) -> bool {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
        .is_ok()
}

struct Line {
    bytes: [u8; 1024],
    len: usize,
}
impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let count = s.len().min(self.bytes.len() - self.len);
        self.bytes[self.len..self.len + count].copy_from_slice(&s.as_bytes()[..count]);
        self.len += count;
        Ok(())
    }
}

fn log(args: fmt::Arguments<'_>) {
    if !take(&LOGS) {
        return;
    }
    let mut line = Line {
        bytes: [0; 1024],
        len: 0,
    };
    let mode = if cfg!(feature = "range-probe") {
        "RANGE_EXPERIMENT"
    } else {
        "LOG_ONLY"
    };
    let _ = writeln!(line, "[gnw-render-probe v4 {mode}] {args}");
    // SVC OutputDebugString is captured by Eden's Debug.Emulated:Info logger.
    unsafe {
        std::arch::asm!(
            "svc #0x27",
            inlateout("x0") line.bytes.as_ptr() => _,
            in("x1") line.len,
            options(nostack)
        );
    }
}

fn state(cmd: Object, update: impl FnOnce(&mut State)) {
    if let Ok(mut states) = STATES.lock() {
        let key = cmd as usize;
        let index = states.iter().position(|s| s.cmd == key);
        let index = match index {
            Some(index) => index,
            None if states.len() < 64 => {
                states.push(State {
                    cmd: key,
                    ..State::default()
                });
                states.len() - 1
            }
            None => {
                if take(&CAP_LOGS) { log(format_args!("command-state capacity reached; unchanged")); }
                return;
            }
        };
        update(&mut states[index]);
    }
}

fn bind(cmd: Object, stage: i32, index: i32, range: Range, uniform: bool) {
    if stage != COMPUTE || !(0..16).contains(&index) {
        return;
    }
    state(cmd, |s| {
        if uniform {
            s.uniform[index as usize] = range;
        } else {
            s.storage[index as usize] = range;
        }
    });
    // Size selects logging only, never a rendering change or a fighter claim.
    if !uniform && range.size == STOCK_GNW_BYTES && take(&BIND_LOGS) {
        log(format_args!(
            "candidate bind cmd={cmd:p} stage={stage} slot={index} gpu={:#x} bytes={}",
            range.address, range.size
        ));
    }
}

unsafe extern "C" fn storage_one(cmd: Object, stage: i32, index: i32, address: u64, size: usize) {
    bind(
        cmd,
        stage,
        index,
        Range {
            address,
            size: size as u64,
        },
        false,
    );
    let original: BindOne = std::mem::transmute(STORAGE_ONE.load(Ordering::Acquire));
    original(cmd, stage, index, address, size);
}
unsafe extern "C" fn uniform_one(cmd: Object, stage: i32, index: i32, address: u64, size: usize) {
    bind(
        cmd,
        stage,
        index,
        Range {
            address,
            size: size as u64,
        },
        true,
    );
    let original: BindOne = std::mem::transmute(UNIFORM_ONE.load(Ordering::Acquire));
    original(cmd, stage, index, address, size);
}
unsafe fn observe_many(
    cmd: Object,
    stage: i32,
    first: i32,
    count: i32,
    ranges: *const Range,
    uniform: bool,
) {
    if stage == COMPUTE
        && first >= 0
        && count >= 0
        && first.checked_add(count).is_some_and(|end| end <= 16)
        && !ranges.is_null()
    {
        for offset in 0..count {
            bind(
                cmd,
                stage,
                first + offset,
                ranges.add(offset as usize).read(),
                uniform,
            );
        }
    }
}
unsafe extern "C" fn storage_many(
    cmd: Object,
    stage: i32,
    first: i32,
    count: i32,
    ranges: *const Range,
) {
    observe_many(cmd, stage, first, count, ranges, false);
    let original: BindMany = std::mem::transmute(STORAGE_MANY.load(Ordering::Acquire));
    original(cmd, stage, first, count, ranges);
}
unsafe extern "C" fn uniform_many(
    cmd: Object,
    stage: i32,
    first: i32,
    count: i32,
    ranges: *const Range,
) {
    observe_many(cmd, stage, first, count, ranges, true);
    let original: BindMany = std::mem::transmute(UNIFORM_MANY.load(Ordering::Acquire));
    original(cmd, stage, first, count, ranges);
}
unsafe extern "C" fn program(cmd: Object, program: *const c_void, stages: i32) {
    if stages & COMPUTE_BIT != 0 {
        state(cmd, |s| s.program = program as usize);
    }
    let original: BindProgram = std::mem::transmute(PROGRAM.load(Ordering::Acquire));
    original(cmd, program, stages);
}
unsafe extern "C" fn begin(cmd: Object) {
    reset(cmd);
    let original: Begin = std::mem::transmute(BEGIN.load(Ordering::Acquire));
    original(cmd);
}
unsafe extern "C" fn dispatch(cmd: Object, x: i32, y: i32, z: i32) {
    let mut observed = None;
    if let Ok(states) = STATES.lock() {
        observed = states.iter().find(|s| s.cmd == cmd as usize && s.storage[2].size == STOCK_GNW_BYTES).copied();
    }
    if let Some(s) = observed {
        if take(&DISPATCH_LOGS) {
            log(format_args!(
                "dispatch cmd={cmd:p} program={:#x} groups={x},{y},{z}",
                s.program
            ));
            describe_dispatch(s.program, s.uniform[0]);
            for (i, r) in s.storage.iter().enumerate().filter(|(_, r)| r.size != 0) {
                log(format_args!(
                    " ssbo[{i}] gpu={:#x} bytes={}",
                    r.address, r.size
                ));
            }
            for (i, r) in s.uniform.iter().enumerate().filter(|(_, r)| r.size != 0) {
                log(format_args!(
                    " ubo[{i}] gpu={:#x} bytes={}",
                    r.address, r.size
                ));
            }
        }
    }
    let original: Dispatch = std::mem::transmute(DISPATCH.load(Ordering::Acquire));
    #[cfg(feature = "range-probe")]
    if let Some(s) = observed {
        if range_probe::dispatch_bounded(cmd, &s, [x, y, z], original) {
            return;
        }
    }
    original(cmd, x, y, z);
}

// Nested command streams can change bindings outside the calls we observe.
// Forget all state; only subsequent explicit bindings can requalify.
unsafe extern "C" fn call_commands(cmd: Object, count: i32, handles: *const u64) {
    reset(cmd);
    let original: unsafe extern "C" fn(Object, i32, *const u64) =
        std::mem::transmute(CALL.load(Ordering::Acquire));
    original(cmd, count, handles);
}
unsafe extern "C" fn copy_commands(cmd: Object, count: i32, handles: *const u64) {
    reset(cmd);
    let original: unsafe extern "C" fn(Object, i32, *const u64) =
        std::mem::transmute(COPY.load(Ordering::Acquire));
    original(cmd, count, handles);
}
fn reset(cmd: Object) {
    if let Ok(mut states) = STATES.lock() {
        states.retain(|s| s.cmd != cmd as usize);
    }
}
unsafe extern "C" fn end(cmd: Object) -> u64 {
    let original: unsafe extern "C" fn(Object) -> u64 =
        std::mem::transmute(END.load(Ordering::Acquire));
    let result = original(cmd);
    reset(cmd);
    result
}
unsafe extern "C" fn finish_cmd(cmd: Object) {
    reset(cmd);
    let original: Begin = std::mem::transmute(FINISH_CMD.load(Ordering::Acquire));
    original(cmd);
}

fn wrap(
    name: &[u8],
    original: *const c_void,
    hook: *const c_void,
    saved: &AtomicUsize,
) -> *const c_void {
    if original.is_null() || original == hook {
        return original;
    }
    match saved.compare_exchange(0, original as usize, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => {
            log(format_args!(
                "resolved {} original={original:p}",
                String::from_utf8_lossy(name)
            ));
            hook
        }
        Err(previous) if previous == original as usize => hook,
        Err(_) => {
            RELIABLE.store(0, Ordering::Release);
            log(format_args!(
                "changed implementation for {}; leaving untouched",
                String::from_utf8_lossy(name)
            ));
            original
        }
    }
}

unsafe extern "C" fn get_proc(device: *const c_void, name: *const c_char) -> *const c_void {
    let original: GetProc = std::mem::transmute(GET_PROC.load(Ordering::Acquire));
    let result = original(device, name);
    if name.is_null() {
        return result;
    }
    range_probe::resolve_getters(device, original);
    let bytes = CStr::from_ptr(name).to_bytes();
    if let Some(replacement) = memory_probe::resolve(bytes, result) {
        return replacement;
    }
    let (hook, saved): (*const c_void, &AtomicUsize) = match bytes {
        b"nvnCommandBufferBindStorageBuffer" => (storage_one as *const c_void, &STORAGE_ONE),
        b"nvnCommandBufferBindStorageBuffers" => (storage_many as *const c_void, &STORAGE_MANY),
        b"nvnCommandBufferBindUniformBuffer" => (uniform_one as *const c_void, &UNIFORM_ONE),
        b"nvnCommandBufferBindUniformBuffers" => (uniform_many as *const c_void, &UNIFORM_MANY),
        b"nvnCommandBufferBindProgram" => (program as *const c_void, &PROGRAM),
        b"nvnCommandBufferDispatchCompute" => (dispatch as *const c_void, &DISPATCH),
        b"nvnCommandBufferEndRecording" => (end as *const c_void, &END),
        b"nvnCommandBufferFinalize" => (finish_cmd as *const c_void, &FINISH_CMD),
        b"nvnCommandBufferBeginRecording" => (begin as *const c_void, &BEGIN),
        b"nvnCommandBufferCallCommands" => (call_commands as *const c_void, &CALL),
        b"nvnCommandBufferCopyCommands" => (copy_commands as *const c_void, &COPY),
        b"nvnDeviceGetProcAddress" => (get_proc as *const c_void, &GET_PROC),
        _ => return result,
    };
    wrap(bytes, result, hook, saved)
}

unsafe extern "C" fn bootstrap(name: *const c_char) -> *const c_void {
    let original: Bootstrap = std::mem::transmute(BOOT.load(Ordering::Acquire));
    let result = original(name);
    if !name.is_null() && CStr::from_ptr(name).to_bytes() == b"nvnDeviceGetProcAddress" {
        wrap(
            b"nvnDeviceGetProcAddress",
            result,
            get_proc as *const c_void,
            &GET_PROC,
        )
    } else {
        result
    }
}

#[skyline::main(name = "gnw_render_probe")]
pub fn main() {
    log(format_args!(
        "loaded; opt-in size-only experiment={}; 512-line lifetime log cap",
        cfg!(feature = "range-probe")
    ));
    let mut address = 0usize;
    let result = unsafe { skyline::nn::ro::LookupSymbol(&mut address, BOOT_NAME.as_ptr().cast()) };
    log(format_args!(
        "LookupSymbol nvnBootstrapLoader result={result:#x} address={address:#x}"
    ));
    if result != 0 || address == 0 {
        log(format_args!("NO HOOK: bootstrap unavailable"));
        return;
    }
    let mut trampoline = std::ptr::null_mut();
    unsafe {
        skyline::hooks::A64HookFunction(
            address as *const c_void,
            bootstrap as *const c_void,
            &mut trampoline,
        );
    }
    BOOT.store(trampoline as usize, Ordering::Release);
    log(format_args!(
        "bootstrap installed trampoline={trampoline:p}; awaiting actual resolver calls"
    ));
}
