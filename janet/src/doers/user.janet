(use ./lib/definers)
(import ../collector)

(defdoer :user
  "Manage Unix users"
  :name-is "User's username"
  (ensure
    (property "gecos" [:string] :mandatory "User's name or description")
    (property "home-dir" [:string] :mandatory "User's home dir")
    (property "primary-group" [:string :number] :mandatory
              "Group name or GID to which user belongs"
              :default "staff")
    (property "shell" [:string] :mandatory "User's shell" :default "/bin/zsh")
    (property "uid" [:number] :mandatory "UID of user")
    (property "other-groups" [:tuple] :optional
              "Group names (`:string`) or GIDs (`:number`) to which user belongs")
    (property "password-hash" [:string] :optional
              "Hash to insert in `/etc/shadow`")
    (property "profiles" [:tuple] :optional
              "List of existing profiles (`:string`)"))

  (notes
    "The actual user management is done via `useradd(8)`, `usermod(8)` and
     `userdel(8)`, so Gurp shares their limitations, such as disallowing
     modification of a logged in user."
    "Removing a group from `other-groups` will not remove the user from that
     group. This is a limitation of usermod(1m)."
    "The doer does not create or otherwise manage the user's home directory."
    "To unlock an account, use a hash of `NP`."
    "You can create non-primary groups for a new user, but not change them for
     an existing one."))

(make-fn :ensure :user)
(make-fn :remove :user)
