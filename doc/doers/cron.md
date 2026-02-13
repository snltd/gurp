# cron

Manage cron jobs. Crontab entries are prefixed with a machine-generated string.

## Resource Name

Convenient name for job. (`:string`)

## cron/ensure

```janet
(cron/ensure "mostly-default-values"
             :minute 6
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
```

```janet
(cron/ensure "lots-of-values"
             :minute 6
             :hour 4
             :day-of-month "*"
             :day-of-week 5
             :label "some-cron-job"
             :user "test-user"
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:command` | `string` | Command which runs |  |
| `:user` | `string` | Username which runs job. Must already exist | `"root"` |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:day-of-month` | `string number` | Day(s) of month on which job runs | `"*"` |
| `:day-of-week` | `string number` | Numeric day(s) on which job runs. 0=Sunday | `"*"` |
| `:hour` | `string number` | Hour(s) at which job runs | `"*"` |
| `:minute` | `string number` | Minute(s) job runs at. Accepts divisions and ranges | `"*"` |
| `:month-of-year` | `string number` | Month(s) in which job runs | `"*"` |

## cron/remove

```janet
(cron/remove "that-old-cron-job")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:user` | `string` | Username which runs job. Must already exist | `"root"` |

### Optional Properties

None

## Notes

- Like other config management tools, Gurp precedes managed lines in the crontab with an identifying string. That string contains the resource ID which, includes the role, resource-type and identifying-name.
- As illumos doesn't have the kind of cron.d support that some other OSes have, Gurp has to use the user's proper crontab, which it does by shelling out to `/bin/crontab`. This gives you crontab's standard value checking: Gurp doesn't check any values itself.
- The doer does not include any kind of user or `cron.allow` management, so you'll have to use other methods to make sure your users are allowed to run the jobs you define.
