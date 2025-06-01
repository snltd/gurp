(role basenode
      (pkg/ensure "helix")
      (pkg/remove "go")
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
