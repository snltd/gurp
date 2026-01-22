(cron/ensure "tightly-specced"
             :minute 6
             :hour 4
             :day-of-month "*"
             :day-of-week 5
             :label "some-cron-job"
             :user "test-user"
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
