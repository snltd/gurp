(cron/ensure "loosely-specced"
             :minute 6
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
