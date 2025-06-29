#[macro_export]
macro_rules! debug {
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        if $opts.debug {
            println!("DEBUG [{}] {}", $component, format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! apply_resources {
    ($summary_total:ident, $changed_ids:ident, $resources:expr, $opts:expr) => {
        for resource in $resources {
            let summary = resource.apply($opts)?;
            $summary_total = $summary_total + summary;
            if summary.changes > 0 {
                $changed_ids.insert(resource.id.clone());
            }
        }
    };
}
