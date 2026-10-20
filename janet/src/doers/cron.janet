(use ./lib)
(import ../collector)

(defdoer :cron
  "Manage cron jobs. Crontab entries are prefixed with a machine-generated string."
  :name-is "Convenient name for job."

  :mandatory-props-ensure
  {:command {:types [:string]
             :help "Command which runs"}
   :user {:types [:string]
          :help "Username which runs job. Must already exist"
          :default "root"}
   :day-of-month {:types [:string :number]
                  :help "Day(s) of month on which job runs"
                  :default "*"}
   :day-of-week {:types [:string :number]
                 :help "Numeric day(s) on  which job runs. 0=Sunday"
                 :default "*"}
   :hour {:types [:string :number]
          :help "Hour(s) at which job runs"
          :default "*"}
   :minute {:types [:string :number]
            :help "Minute(s) job runs at. Accepts divisions and ranges"
            :default "*"}
   :month-of-year {:types [:string :number]
                   :help "Month(s) in which job runs"
                   :default "*"}}

  :mandatory-props-remove {:user {:types [:string]
                                  :help "Username which runs job. Must exist"
                                  :default "root"}}

  :notes
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

(make-ensure-fn "cron")
(make-remove-fn "cron")
