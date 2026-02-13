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
  ["Like other config management tools, Gurp precedes managed lines in the
    crontab with an identifying string. That string contains the resource ID
    which, includes the role, resource-type and identifying-name."
   "As illumos doesn't have the kind of cron.d support that some other OSes
    have, Gurp has to use the user's proper crontab, which it does by shelling
    out to `/bin/crontab`. This gives you crontab's standard value checking:
    Gurp doesn't check any values itself."
   "The doer does not include any kind of user or `cron.allow` management, so
    you'll have to use other methods to make sure your users are allowed to run
    the jobs you define."])
