(use judge)
(use ../../src/collector)
(use ../../src/user-helpers)
(import ../../src/doers/cron)

(deftest "cron-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (cron/ensure "loosely-specced"
               :minute 6
               :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))

  (cron/ensure "tightly-specced"
               :minute 6
               :hour 4
               :day-of-month "*"
               :day-of-week 5
               :label "some-cron-job"
               :user "test-user"
               :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))

  (cron/remove "that-old-cron-job")

  (test *collector*
    @{:ensure @{:cron @[{:_id "/test-role/cron/loosely-specced"
                         :command "/bin/thing arg1 arg2 arg3"
                         :day-of-month "*"
                         :day-of-week "*"
                         :hour "*"
                         :minute 6
                         :month-of-year "*"
                         :name "loosely-specced"
                         :role "test-role"
                         :user "root"}
                        {:_id "/test-role/cron/some-cron-job"
                         :command "/bin/thing arg1 arg2 arg3"
                         :day-of-month "*"
                         :day-of-week 5
                         :hour 4
                         :label "some-cron-job"
                         :minute 6
                         :month-of-year "*"
                         :name "tightly-specced"
                         :role "test-role"
                         :user "test-user"}]}
      :remove @{:cron @[{:_id "/test-role/cron/that-old-cron-job"
                         :name "that-old-cron-job"
                         :role "test-role"}]}}))

(deftest "cron-error"
  (test-error
    (cron/ensure "missing-data" :hour 6)
    "did not find mandatory property :command. Mandatory properties are :command")

  (test-error
    (cron/ensure "junk-keys"
                 :command "/bin/effort"
                 :minute 1
                 :day "monday"
                 :colour "blue"
                 :hour 6)
    "unexpected property :colour. Valid properties are :command, :minute, :hour, :month-of-year, :day-of-month, :user, :label, :day-of-week"))
