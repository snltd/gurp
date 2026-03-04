(cron/ensure "root-cron-job"
             :minute 6
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
