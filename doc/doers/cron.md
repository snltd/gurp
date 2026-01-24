# cron

Manage cron jobs. Crontab entries are prefixed with a                  machine-generated string.

## Resouce Name

Convenient name for job. (`:string`)

## cron/ensure

```janet
(cron/ensure "loosely-specced"
             :minute 6
             :command (argcat "/bin/thing" "arg1" "arg2" "arg3"))
```

```janet
(cron/ensure "tightly-specced"
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

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:day-of-month` | `string number` | Day(s) of month on which job runs | `"*"` |
| `:day-of-week` | `string number` | Numeric day(s) on  which job runs. 0=Sunday | `"*"` |
| `:hour` | `string number` | Hour(s) at which job runs | `"*"` |
| `:minute` | `string number` | Minute(s) job runs at. Accepts divisions and ranges | `"*"` |
| `:month-of-year` | `string number` | Month(s) in which job runs | `"*"` |
| `:user` | `string` | Username which runs job. Must already exist | `"root"` |

## cron/remove

```janet
(cron/remove "that-old-cron-job")
```

### Mandatory Properties

None

### Optional Properties

None

