#![feature(naked_functions)]
use std::arch::asm;
const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;


pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64, // Stack pointer register, points to the top of the stack
    r15: u64, // General purpose register
    r14: u64, // General purpose register
    r13: u64, // General purpose register
    r12: u64, // General purpose register
    rbx: u64, // Base register, used as a pointer to data in the DS segment
    rbp: u64, // Base pointer register, used to point to the base of the stack frame
}

impl Thread  {
    fn new() -> Self {
        Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        let base_thread = Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        let mut threads = vec![base_thread];
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0,
        }
    }

    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0)
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

// [Available] - Employee is at their desk but not working.
// [Running]   - Employee is actively working.
// [Ready]     - Employee is ready to start working.

// Step 1: [Current Employee] -> Looking who's next to work.
// Step 2: [Looking for Ready] -> Manager checks each desk in a circle.
// Step 3: [Switching Tasks]   -> Ready employee is found and set to Running.
// Step 4: [Changing Focus]    -> Manager focuses on the new active employee.
// Step 5: [The Actual Switch] -> Magic notebook transfer of work progress.
// Step 6: [Continuation]      -> Workday continues if employees are present.

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current; // step 1
        while self.threads[pos].state != State::Ready { // step 2
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
            let new: *const ThreadContext = &self.threads[pos].ctx;
            asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C"));
        }
        self.threads.len() > 0

    }
}



fn main() {
    println!("Hello, world!");
}
