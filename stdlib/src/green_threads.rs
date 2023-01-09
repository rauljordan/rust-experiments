use core::arch::asm;

static mut RUNTIME: usize = 0;
const MAX: isize = 48;
const STACK_SIZE: usize = 1024 * 1024 * 2;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

pub struct Thread {
    id: usize,
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

impl Thread {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            stack: vec![0u8; STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum State {
    Available,
    Running,
    Ready,
}

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

impl Runtime {
    pub fn new(max_threads: usize) -> Self {
        let base = Thread {
            id: 0,
            stack: vec![0u8; STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };
        let mut threads = vec![base];
        let available = (1..max_threads)
            .map(|i| Thread::new(i))
            .collect::<Vec<Thread>>();
        threads.extend(available);
        Self {
            threads,
            current: 0,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0);
    }

    pub fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    pub fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no threads available");
        let size = available.stack.len();
        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }
        available.state = State::Ready;
    }

    #[inline(never)]
    pub fn t_yield(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *mut ThreadContext = &mut self.threads[pos].ctx;
            asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C"));
        }
        self.threads.len() > 0
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    }
}

#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn))
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch() {
    asm!(
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    );
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    };
}

fn nyan() -> ! {
    println!("nyan");
    loop {}
}

use core::arch::asm;

const MAX_DEPTH: isize = 48;
const STACK_SIZE: usize = 1024 * 1024 * 2;

#[derive(Debug, Default)]
#[repr(C)]
struct StackContext {
    rsp: u64,
}

fn nyan() -> ! {
    println!("nyan nyan nyan");
    loop {}
}

pub fn move_to_nyan() {
    let mut ctx = StackContext::default();
    let mut stack = vec![0u8; MAX as usize];
    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(MAX_DEPTH);
        let aligned = (stack_bottom as usize & !15) as *mut u8;
        std::ptr::write(aligned.offset(-16) as *mut u64, nyan as u64);
        ctx.rsp = aligned.offset(-16) as u64;
        gt_switch(&mut ctx);
    }
}

unsafe fn gt_switch(new: *const StackContext) {
    asm!(
        "mov rsp, [{0} + 0x00]",
        "ret",
        in(reg) new,
    )
}
