macro_rules! zone_attr {
    ($name:expr, $type:expr, $value:expr) => {
        format!(
            "add attr\n\tset name={}\n\tset type={}\n\tset value={}\nend\n",
            $name, $type, $value
        )
    };
}
