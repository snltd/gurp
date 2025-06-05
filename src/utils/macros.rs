#[macro_export]
macro_rules! info {
    ($opts:expr, $($arg:tt)*) => {
        if $opts.verbose || $opts.noop || $opts.debug {
            println!("{}", format!($($arg)*).bold());
        } else {
            println!($($arg)*);
        }
    };
}

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
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        if $opts.debug {
            println!("DEBUG [{}] {}", $component, format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        println!("{}", format!("WARN [{}] {}", $component, format!($($arg)*)).red())
    };
}

#[macro_export]
macro_rules! error {
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        eprintln!("{}", format!("ERROR [{}] {}", $component, format!($($arg)*)).bold().red())
    };
}

#[macro_export]
macro_rules! unpack_fn {
    ($suffix:ident, $enum_variant:ident, $ty:ty) => {
        paste! {
            pub fn [<unpack_ $suffix>](
                resource_list: &JanetArray,
                _opts: &Opts,
            ) -> anyhow::Result<Vec<Resource>> {
                resource_list
                    .iter()
                    .map(|r| {
                        let val = <$ty>::try_from(r)?;
                        Ok(Resource::$enum_variant(val))
                    })
                    .collect()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_apply {
    ($ty:ty) => {
        paste! {
            impl Apply for $ty {
                fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
                    let output = Output::new(&self.doer, opts);
                    match self.action {
                        Action::Ensure => self.apply_ensure(opts, &output),
                        Action::Remove => self.apply_remove(opts, &output),
                    }
                }
            }
        }
    };
}
