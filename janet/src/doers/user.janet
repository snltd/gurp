(use ./lib)
(import ../collector)

(defdoer :user
  "Manage Unix users"
  :name-is "User's username"

  :mandatory-props-ensure
  {:gecos {:types [:string]
           :help "User's name or description"}
   :home-dir {:types [:string]
              :help "User's home dir"}
   :primary-group {:types [:string :number]
                   :help "Group name or GID to which user belongs"}
   :shell {:types [:string]
           :help "User's shell"}
   :uid {:types [:number]
         :help "UID of user"}}

  :optional-props-ensure
  {:other-groups {:types [:tuple]
                  :help "Group names (:string) or GIDs (:number) to which user belongs"}
   :password-hash {:types [:string]
                   :help "Hash to insert in /etc/shadow"}
   :profiles {:types [:tuple]
              :help "List of existing profiles (:string)"}}

  :defaults-ensure
  {:shell "/bin/zsh"
   :primary-group "staff"}

  :notes
  ["The actual user management is done via `useradd(8)`, `usermod(8)` and
    `userdel(8)`, so Gurp shares their limitations, such as disallowing
    modification of a logged in user."
   "Removing a group from `other-groups` will not remove the user from that
    group. This is a limitation of usermod(1m)."
   "The doer does not create or otherwise manage the user's home directory."
   "To unlock an account, use a hash of `NP`."
   "You can create non-primary groups for a new user, but not change them for
    an existing one."])

(defensure "user")
(defremove "user")
