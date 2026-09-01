(use judge)
(use ./test-lib)
(use ../../src/collector)
(use ../../src/dsl)
(import ../../src/doers/cron)

(deftest cron
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "cron")

  (test *collector*
    @{:ensure @{:cron @[{:_id "/test-role/cron/print-cron-job"
                         :command "/bin/thing arg1 arg2 arg3"
                         :day-of-month "*"
                         :day-of-week 5
                         :hour 4
                         :label "print-cron-job"
                         :minute 6
                         :month-of-year "*"
                         :name "lots-of-values"
                         :role "test-role"
                         :user "lp"}
                        {:_id "/test-role/cron/root-cron-job"
                         :command "/bin/thing arg1 arg2 arg3"
                         :day-of-month "*"
                         :day-of-week "*"
                         :hour "*"
                         :minute 6
                         :month-of-year "*"
                         :name "root-cron-job"
                         :role "test-role"
                         :user "root"}]}
      :remove @{:cron @[{:_id "/test-role/cron/that-old-cron-job"
                         :name "that-old-cron-job"
                         :role "test-role"
                         :user "root"}]}}))

(deftest cron-error
  (test-error
    (cron/ensure "missing-data" :hour 6)
    "In cron/ensure missing-data: did not find mandatory property :command. Mandatory properties are :command, :user")

  (test-error
    (cron/ensure "junk-keys"
                 :command "/bin/effort"
                 :minute 1
                 :day "monday"
                 :colour "blue"
                 :hour 6)
    "In cron/ensure junk-keys: unexpected property :colour. Valid properties are :command, :user, :minute, :hour, :month-of-year, :day-of-month, :label, :day-of-week"))
