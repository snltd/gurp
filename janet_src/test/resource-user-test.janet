(use judge)
(use ../lib/gurp)

(deftest "user-resources"
  (set *collector* (new-collector))

  (setdyn :role-dyn "test-role")
  (user/ensure "rob"
               :uid 264
               :primary-group "sysadmin"
               :home-dir "/home/rob"
               :shell "/bin/zsh"
               :gecos "Test User"
               :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui")

  (user/remove "lolex")

  (test *collector*
        @{:ensure @{:user @[{:_id "/test-role/user/rob"
                             :gecos "Test User"
                             :home-dir "/home/rob"
                             :name "rob"
                             :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui"
                             :primary-group "sysadmin"
                             :role "test-role"
                             :shell "/bin/zsh"
                             :uid 264}]}
          :remove @{:user @[{:_id "/test-role/user/lolex"
                             :name "lolex"
                             :role "test-role"}]}}))

(deftest "user-errors"
  (test-error
    (user/ensure "wat"
                 :uid 100)
    "Failed to validate user input for user 'wat' : user missing required key(s): home-dir, primary-group, gecos, shell")

  (test-error
    (user/ensure "rob"
                 :uid 264
                 :hair "reddish"
                 :height "quite-tall"
                 :primary-group "sysadmin"
                 :home-dir "/home/rob"
                 :shell "/bin/zsh"
                 :gecos "Test User"
                 :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui")
    "Failed to validate user input for user 'rob' : user 'rob' has unrecognised key(s): height, hair"))
