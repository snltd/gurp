(use judge)
(use ../lib/gurp)

(deftest "test cron functions"
  (setdyn :role-dyn "test-role")
  (test
    (cron/ensure "loosely-specced"
                 :minute 6
                 :cmd (argcat "command" "arg1" "arg2" "arg3"))
    {:cron {:_id "/test-role/cron/loosely-specced"
            :action :ensure
            :cmd "command arg1 arg2 arg3"
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
                 :cmd (argcat "command" "arg1" "arg2" "arg3"))
    {:cron {:_id "/test-role/cron/some-cron-job"
            :action :ensure
            :cmd "command arg1 arg2 arg3"
            :day-of-month "*"
            :day-of-week 5
            :hour 4
            :label "some-cron-job"
            :minute 6
            :month-of-year "*"
            :name "tightly-specced"
            :role "test-role"
            :user "test-user"}})

  (test
    (cron/remove "that-old-cron-job")
    {:cron {:_id "/test-role/cron/that-old-cron-job"
            :action :remove
            :name "that-old-cron-job"
            :role "test-role"}}))
