(import ./globals)

(defn site-cron
  "Given a script name and optional args, returns a command string which
  executes said script in site-bin and logs to script-name-log in cron-log-dir"
  [cmd-bin & args]
  (argcat
    (pathcat globals/site-bin cmd-bin)
    (splice args)
    ">"
    (pathcat globals/cron-log-dir (string cmd-bin ".log"))
    "2>&1"))

(defn ip-of
  "Gives you the IP address of the thing called name"
  [name &keys {:with-netmask with-netmask}]
  (def ip (get globals/hostname->ip (keyword name) "0.0.0.0"))
  (if with-netmask (string ip "/" globals/netmask) ip))

(defn ensure-sysdef-publisher
  "Consistently add our local publisher"
  []
  (publisher/ensure "sysdef"
                    :uri globals/pkg-url))

(defn num-field-sort
  "Sort a file when each line's first field is numeric"
  [str]
  (int/u64 (first (fields str))))

