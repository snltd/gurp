(role basenode
      (pkg/ensure "helix")
      (pkg/remove "go")
      (user/ensure "tester"
        :gcos "Testy McTestface"
        :uid 280
        :group "sysadmin"
        :other-groups ["staff" "users"]
        :home-dir "/export/home/tester"
        :shell "/bin/ksh")
      (file/ensure "basenode_file"
                   :path "/tmp/basenode.txt"
                   :content "some words")
      (directory/ensure "merp"
                  :path "/tmp/merp"
                  :mode "0755")
      (directory/remove "junk"
                  :path "/tmp/junk"))

(role devtools
      (pkg/ensure "rust")
      (pkg/ensure "git")
      (file/ensure "git_config"
                   :path "/tmp/git-config.txt"
                   :source "git-config"))

(host "example"
      (basenode)
      (devtools))
