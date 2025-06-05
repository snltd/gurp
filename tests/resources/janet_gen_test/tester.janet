(role basenode
      (pkg/ensure "helix")
      (pkg/remove "go")
      (user/ensure "tester"
                   :gecos "Testy McTestface"
                   :uid 280
                   :group "sysadmin"
                   :other-groups ["staff" "users"]
                   :home-dir "/export/home/tester"
                   :shell "/bin/ksh")
      (file/ensure "/tmp/basenode.txt"
                   :content "some words")
      (directory/ensure "/tmp/merp"
                        :mode "0755")
      (directory/remove "/tmp/junk"))

(role devtools
      (pkg/ensure "rust")
      (pkg/ensure "git")
      (file/ensure "/tmp/git-config.txt"
                   :source "git-config"))

(host "example"
      (basenode)
      (devtools))
