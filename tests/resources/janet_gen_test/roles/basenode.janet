(role basenode
      (package/ensure "helix")
      (package/remove "go")
      (file/ensure "basenode_file"
                   :path "/tmp/basenode.txt"
                   :content "some words")
      (directory/ensure "merp"
                  :path "/tmp/merp"
                  :owner :user/rob/uid
                  :group :user/rob/group
                  :mode "0755")
      (directory/remove "junk"
                  :path "/tmp/junk"))
