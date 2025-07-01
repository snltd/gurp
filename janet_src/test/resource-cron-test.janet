(use judge)
(use ../lib/gurp)

(deftest "test cron functions"
  (setdyn :role-dyn "test-role")
  (test
    (cron/ensure "loosely-specced"
                 :minute 6
                 :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
    {:cron {:_id "/test-role/cron/loosely-specced"
            :action :ensure
            :command "/bin/thing arg1 arg2 arg3"
            :day-of-month "*"
            :day-of-week "*"
            :hour "*"
            :minute 6
            :month-of-year "*"
            :name "loosely-specced"
            :role "test-role"
            :user "root"}})

  (test
    (cron/ensure "tightly-specced"
                 :minute 6
                 :hour 4
                 :day-of-month "*"
                 :day-of-week 5
                 :label "some-cron-job"
                 :user "test-user"
                 :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
    {:cron {:_id "/test-role/cron/some-cron-job"
            :action :ensure
            :command "/bin/thing arg1 arg2 arg3"
            :day-of-month "*"
            :day-of-week 5
            :hour 4
            :label "some-cron-job"
            :minute 6
            :month-of-year "*"
            :name "tightly-specced"
            :role "test-role"
            :user "test-user"}})

  (test-error
    (cron/ensure "missing-data" :hour 6)
    "cron missing required key(s): command")

  (test-error
    (cron/ensure "junk-keys"
                 :command "/bin/effort"
                 :minute 1
                 :day "monday"
                 :colour "blue"
                 :hour 6)
    "cron 'junk-keys' has unrecognised key(s): day, colour")

  (test
    (cron/remove "that-old-cron-job")
    {:cron {:_id "/test-role/cron/that-old-cron-job"
            :action :remove
            :name "that-old-cron-job"
            :role "test-role"}}))
