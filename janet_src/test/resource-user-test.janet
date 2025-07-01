(use judge)
(use ../lib/gurp)

(deftest "test user functions"
  (setdyn :role-dyn "test-role")
  (test
    (user/ensure "rob"
                 :uid 264
                 :primary-group "sysadmin"
                 :home-dir "/home/rob"
                 :shell "/bin/zsh"
                 :gecos "Test User"
                 :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui")
    {:user {:_id "/test-role/user/rob"
            :action :ensure
            :gecos "Test User"
            :home-dir "/home/rob"
            :name "rob"
            :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui"
            :primary-group "sysadmin"
            :role "test-role"
            :shell "/bin/zsh"
            :uid 264}})

  (test-error
    (user/ensure "wat"
                 :uid 100)
    "user missing required key(s): primary-group, home-dir, shell, gecos")

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
    "user 'rob' has unrecognised key(s): height, hair")

  (test
    (user/remove "lolex")
    {:user {:_id "/test-role/user/lolex"
            :action :remove
            :name "lolex"
            :role "test-role"}}))
