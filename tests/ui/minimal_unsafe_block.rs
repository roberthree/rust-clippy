#![warn(clippy::minimal_unsafe_block)]
// #![forbid(unused_unsafe)]

fn safe_fn<T>(x: T) -> T {
    x
}

unsafe fn unsafe_fn<T>(x: T) -> T {
    x
}

struct A;

impl A {
    fn safe_method(self) -> Self {
        self
    }

    unsafe fn unsafe_method(self) -> Self {
        self
    }
}

fn lint_example() {
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers statements
        //~| NOTE: `-D clippy::minimal-unsafe-block` implied by `-D warnings`
        //~| HELP: to override `-D warnings` add `#[allow(clippy::minimal_unsafe_block)]`
        let x = Some(true);
        let y = x.unwrap_unchecked();
    }
}

fn covers_statements() {
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers statements
        let y = unsafe_fn(0);
    };

    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers statements
        unsafe_fn(0);
    };

    unsafe { unsafe_fn(0) };
}

fn covers_array() {
    let x = unsafe { [unsafe_fn(0)] };
    //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily an array

    let x = [unsafe { unsafe_fn(0) }];
}

fn covers_block() {
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a block
        {
            unsafe_fn(0);
        }
    };

    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a block
        unsafe { unsafe_fn(0) }
    };
}

fn covers_closure() {
    let c = unsafe { |x: usize| unsafe_fn(x) };
    //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a closure

    let c = |x: usize| unsafe { unsafe_fn(x) };
}

fn covers_if() {
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily an `if` block
        if unsafe_fn(0) == 0 { safe_fn(0) } else { safe_fn(0) }
    };

    if unsafe { unsafe_fn(0) } == 0 {
        safe_fn(0)
    } else {
        safe_fn(0)
    };
}

fn covers_loop() {
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a `loop` block
        loop {
            unsafe_fn(0);
            break;
        }
    };

    loop {
        unsafe { unsafe_fn(0) };
        break;
    }
}

fn covers_tuple() {
    let x = unsafe { (unsafe_fn(0),) };
    //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a tuple

    let x = (unsafe { unsafe_fn(0) },);
}

fn covers_safe_call() {
    unsafe { safe_fn(unsafe_fn(0)) };
    //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a safe call

    unsafe { unsafe_fn(safe_fn(0)) };

    #[allow(clippy::redundant_closure_call)]
    unsafe {
        //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a safe call
        (|x| unsafe_fn(x))(0)
    };

    #[allow(clippy::redundant_closure_call)]
    (|x| unsafe { unsafe_fn(x) })(0);
}

fn covers_safe_method_call() {
    unsafe { (A {}).safe_method().unsafe_method().safe_method() };
    //~^ ERROR: this `unsafe` block is not minimal as it covers unnecessarily a safe method call

    unsafe { (A {}).safe_method().unsafe_method() }.safe_method();
}
