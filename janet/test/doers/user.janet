(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/user)

(deftest user
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "user" (curenv))

  (test *collector*
    @{:ensure @{:user @[{:_id "/test-role/user/gurpuser"
                         :gecos "Gurp Managed User"
                         :home-dir "/home/gurpuser"
                         :name "gurpuser"
                         :password-hash "w0934cm-4i5c-42u5cn492hrc97h234ui"
                         :primary-group "sysadmin"
                         :role "test-role"
                         :shell "/bin/zsh"
                         :uid 1264}]}
      :remove @{:user @[{:_id "/test-role/user/lolex"
                         :name "lolex"
                         :role "test-role"}]}}))

(deftest user-error
  (test-error
    (user/ensure "wat"
                 :uid 100)
    "In user/ensure wat: did not find mandatory property :home-dir. Mandatory properties are :home-dir, :primary-group, :uid, :gecos, :shell")

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
    "In user/ensure rob: unexpected property :height. Valid properties are :home-dir, :primary-group, :uid, :gecos, :shell, :other-groups, :password-hash, :profiles, :label"))
