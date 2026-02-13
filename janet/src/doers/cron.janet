(use ./lib)
(import ../collector)

(def doer :cron)
(def description "Manage cron jobs. Crontab entries are prefixed with a
                  machine-generated string.")
(def name-is "Convenient name for job.")
(def mandatory-props-ensure
  {:command {:types [:string]
             :help "Command which runs"}
  :user {:types [:string]
          :help "Username which runs job. Must already exist"}})
(def optional-props-ensure
  {:day-of-month {:types [:string :number]
                  :help "Day(s) of month on which job runs"}
   :day-of-week {:types [:string :number]
                 :help "Numeric day(s) on  which job runs. 0=Sunday"}
   :hour {:types [:string :number]
          :help "Hour(s) at which job runs"}
   :minute {:types [:string :number]
            :help "Minute(s) job runs at. Accepts divisions and ranges"}
   :month-of-year {:types [:string :number]
                   :help "Month(s) in which job runs"}})

(def mandatory-props-remove
  {:user {:types [:string]
          :help "Username which runs job. Must already exist"}})
(def optional-props-remove {})

(def defaults-ensure
  {:hour "*"
   :minute "*"
   :day-of-month "*"
   :day-of-week "*"
   :month-of-year "*"
   :user "root"})
(def defaults-remove
  {:user "root"})

(defn ensure
  "Given a cron job name and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a cron job name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Jobs are added and removed via `crontab(1)`, so no invalid entries should
  end up being applied. Gurp-managed cron jobs are prefixed with a comment
  line"])
