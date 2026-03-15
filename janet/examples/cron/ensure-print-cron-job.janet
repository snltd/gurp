(cron/ensure "lots-of-values"
             :minute 6
             :hour 4
             :day-of-month "*"
             :day-of-week 5
             :label "print-cron-job"
             :user "lp"
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
