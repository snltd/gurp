#[macro_export]
macro_rules! verbose {
    ($opts:expr, $($arg:tt)*) => {
        if $opts.verbose || $opts.noop || $opts.debug {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($opts:expr, $($arg:tt)*) => {
        if $opts.debug {
            println!($($arg)*);
        }
    };
}
