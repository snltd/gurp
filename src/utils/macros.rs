/*
#[macro_export]
macro_rules! no_change {
    ($opts:expr) => {
        verbose!($opts, "[{}/{}] NO CHANGE", $self_.doer, $self_.name);
    };
}

#[macro_export]
macro_rules! not_there {
    ($opts:expr) => {
        debug!($opts, "[{}/{}] DOES NOT EXIST", $self_.doer, $self_.name);
    };
}

#[macro_export]
macro_rules! creating {
    println!("[{}/{}] CREATING", $self_.doer, $self_.name);
}

#[macro_export]
macro_rules! change {
    ($self_:ident, $current:ident, $property:ident) => {
        let current_val = match $current.$property {
            Some(val) => val,
            None => "<none>".to_owned(),
        };

        let desired_val = match $self_.desired_state.$property {
            Some(val) => val,
            None => "<none>".to_owned(),
        };

        println!(
            "[{}/{}] CHANGE {} '{}' -> '{}'",
            $self_.doer,
            $self_.name,
            stringify!($property),
            current_val,
            desired_val,
        );
    };
}
*/

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

/*
#[macro_export]
macro_rules! generate_unpack_functions {
    ($resource_type:ident) => {
        paste::paste! {
            pub fn unpack_ensure_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Ensure>> {
                resource_list
                    .iter()
                    .map(|r| {
                        let resource = [<$resource_type ToEnsure>]::try_from(r)?;
                        Ok(Ensure::$resource_type(resource))
                    })
                    .collect()
            }

            pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
                resource_list
                    .iter()
                    .map(|r| {
                        let resource = [<$resource_type ToRemove>]::try_from(r)?;
                        Ok(Remove::$resource_type(resource))
                    })
                    .collect()
            }
        }
    };
}
    */
