# user

Manage Unix users

## Resource Name

User's username (`:string`)

## user/ensure

```janet
(user/ensure "gurpuser"
             :uid 1264
             :primary-group "sysadmin"
             :home-dir "/home/gurpuser"
             :shell "/bin/ksh"
             :gecos "Gurp Managed User"
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

## Notes

- The actual user management is done via `useradd(8)`, `usermod(8)` and `userdel(8)`, so Gurp shares their limitations, such as disallowing modification of a logged in user.
- Removing a group from `other-groups` will not remove the user from that group. This is a limitation of usermod(1m).
- The doer does not create or otherwise manage the user's home directory.
- To unlock an account, use a hash of `NP`.
- You can create non-primary groups for a new user, but not change them for an existing one.
