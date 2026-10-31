(use ./definers)
(use ./validators)
(import ../collector)

(defdoer :cron
  "Manage cron jobs. Crontab entries are prefixed with a machine-generated string."
  :name-is "Convenient name for job."

  (ensure
    (property "command" [:string] :mandatory "Command which runs")
    (property "user" [:string] :mandatory
      "Username which runs job. Must already exist"
      :default "root")
    (property "day-of-month" [:string :number] :mandatory
      "Day(s) of month on which job runs"
      :default "*")
    (property "day-of-week" [:string :number] :mandatory
      "Numeric day(s) on  which job runs. 0=Sunday"
      :default "*")
    (property "hour" [:string :number] :mandatory "Hour(s) at which job runs"
      :default "*")
    (property "minute" [:string :number] :mandatory
      "Minute(s) job runs at. Accepts divisions and ranges"
      :default "*")
    (property "month-of-year" [:string :number] :mandatory
      "Month(s) in which job runs"
      :default "*"))

  (remove
    (property "user" [:string] :mandatory "Username which runs job. Must exist"
      :default "root"))

  (notes
    "Like other config management tools, Gurp precedes managed lines in the
    crontab with an identifying string. That string contains the resource ID
    which, includes the role, resource-type and identifying-name."
    "As illumos doesn't have the kind of cron.d support that some other OSes
    have, Gurp has to use the user's proper crontab, which it does by shelling
    out to `/bin/crontab`. This gives you crontab's standard value checking:
    Gurp doesn't check any values itself."
    "The doer does not include any kind of user or `cron.allow` management, so
    you'll have to use other methods to make sure your users are allowed to run
    the jobs you define."))

(make-fn :ensure :cron)
(make-fn :remove :cron)
