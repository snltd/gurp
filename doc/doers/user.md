# user

Manage Unix users

## Resouce Name

User's username (`:string`)

## user/ensure

```janet
(user/ensure "rob"
             :uid 264
             :primary-group "sysadmin"
             :home-dir "/home/rob"
             :shell "/bin/zsh"
             :gecos "Test User"
             :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gecos` | `string` | User's name or description |  |
| `:home-dir` | `string` | User's home dir |  |
| `:primary-group` | `string number` | Group name or GID to which user belongs | `"staff"` |
| `:shell` | `string` | User's shell | `"/bin/zsh"` |
| `:uid` | `number` | UID of user |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:other-groups` | `tuple` | Group names (:string) or GIDs (:number) to which user belongs |  |
| `:password-hash` | `string` | Hash to insert in /etc/shadow |  |
| `:profiles` | `tuple` | List of existing profiles (:string) |  |

## user/remove

```janet
(user/remove "lolex")
```

### Mandatory Properties

None

### Optional Properties

None

