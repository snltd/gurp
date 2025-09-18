(import ../globals)

(def snap-bin (pathcat globals/site-bin "zfs-snap"))
(def regular-omit `"rpool/zones/*,*/logs,*/var*"`)

(role zfs-snapshot
      (file/ensure snap-bin
                   :mode "0755"
                   :from "/home/rob/.cargo/bin/zfs-snap")

      (cron/ensure "zfs-daily-snapshots"
                   :minute 0
                   :hour 12
                   :command (argcat snap-bin
                                    "--type=day"
                                    (string "--omit=" regular-omit)
                                    ">"
                                    (pathcat globals/cron-log-dir "zfs-daily-snapshots.log")
                                    "2>&1"))

      (cron/ensure "zfs-monthly-snapshots"
                   :minute 0
                   :hour 12
                   :day-of-week 1
                   :command (argcat snap-bin
                                    "--type=month"
                                    (string "--omit=" regular-omit)
                                    ">"
                                    (pathcat globals/cron-log-dir "zfs-monthly-snapshots.log")
                                    "2>&1"))

      (cron/ensure "zfs-work-snapshots"
                   :minute "*/10"
                   :command (argcat snap-bin
                                    "--type=time"
                                    (zfscat globals/fast-pool "/export/home/rob/work")
                                    ">"
                                    (pathcat globals/cron-log-dir "zfs-work-snapshots.log")
                                    "2>&1"))

      (cron/ensure "zfs-important-snapshots"
                   :minute "0,30"
                   :command (argcat snap-bin
                                    "--type=time"
                                    "--recurse"
                                    (zfscat globals/fast-pool "/export/home")
                                    (zfscat globals/big-pool "/export/flac")
                                    `"*/data*"`
                                    `"*/build*"`
                                    ">"
                                    (pathcat globals/cron-log-dir "zfs-important-snapshots.log")
                                    "2>&1")))
