(role basenode
      :package "helix"
      :file {:name "basenode_file"
             :path "/tmp/basenode.txt"
             :content "some words"}
      :directory {:name "merp"
                  :path "/tmp/merp"
                  :owner :user/rob/uid
                  :group :user/rob/group
                  :mode "0755"})
