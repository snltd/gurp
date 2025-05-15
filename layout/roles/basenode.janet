(import ../vars)
(use ../lib/helpers)
(use ../defaults)

(role basenode
      :packages [(ensure "helix")]
      :users [(ensure "rob"
                      :uid 264
                      :gid 14
                :gcos vars/gibbus
                      :dir "/home/rob")]
      :files [(ensure "basenode_file"
                      :path "/tmp/basenode.txt"
                      :content "some words")]
      :directories [(ensure "merp"
                            :path "/tmp/merp"
                            :owner :user/rob/uid
                            :group :user/rob/group
                            :mode "0755")])
