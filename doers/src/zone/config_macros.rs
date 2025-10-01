macro_rules! zone_attr {
    ($name:expr, $type:expr, $value:expr) => {
        format!(
            "add attr\n\tset name={}\n\tset type={}\n\tset value={}\nend\n",
            $name, $type, $value
        )
        .as_str()
    };
}

macro_rules! zone_capped_memory {
    ($cap:expr) => {
        format!(
            "add capped-memory\n\tset physical={}\n\tset swap={}\nend\n",
            $cap.physical, $cap.swap,
        )
        .as_str()
    };
}

macro_rules! zone_dataset {
    ($name:expr) => {
        format!("add dataset\n\tset name={}\nend\n", $name).as_str()
    };
}

macro_rules! zone_device {
    ($path:expr) => {
        format!("add device\n\tset match={}\nend\n", $path).as_str()
    };
}

macro_rules! zone_fs {
    ($fs:expr) => {{
        let mut ret = format!(
            "add fs\n\tset dir={}\n\tset special={}\n\tset type={}\n",
            $fs.dir, $fs.special, $fs.fs_type
        );

        if let Some(options) = &$fs.options {
            ret.push_str(&format!("\tset options={}\n", options.join(",")));
        }

        ret.push_str("end\n");
        ret
    }};
}

macro_rules! zone_rctl {
    ($rctl:expr) => {
        format!(
            "add rctl\n\tset name={}\n\tset value=(priv={},limit={},action={})\nend\n",
            $rctl.name, $rctl.rctl_priv, $rctl.limit, $rctl.action
        )
        .as_str()
    };
}
