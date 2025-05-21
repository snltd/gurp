(use ../../../lib/gurp)

(role devtools
      (package/ensure "rust")
      (package/ensure "git")
      (file/ensure "git_config"
                   :path "/tmp/git-config.txt"
                   :source "git-config"))
