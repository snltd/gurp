(use judge)
(use ../lib/gurp)

(test-macro
  (host "example-node" (role "basenode") (role "devtools"))
  (upscope
    (setdyn :host-dyn (string "example-node"))
    (defn machine-config
      []
      (role "basenode")
      (role "devtools")
      {:metadata {:name "example-node"} :resources (finalise *collector*)})))

(test-macro
  (role basenode
        (pkg/ensure "helix")
        (pkg/remove "go")
        (file/ensure "/tmp/basenode.txt"
                     :content "some words")
        (directory/ensure "/tmp/merp"
                          :owner "tester"
                          :group "tester"
                          :mode "0755")
        (directory/remove "/tmp/junk"))
  (defn basenode
    []
    (setdyn :role-dyn (string (quote basenode)))
    (pkg/ensure "helix")
    (pkg/remove "go")
    (file/ensure "/tmp/basenode.txt" :content "some words")
    (directory/ensure "/tmp/merp" :owner "tester" :group "tester" :mode "0755")
    (directory/remove "/tmp/junk")))

(test-macro
  (section "test-section"
           (svc/ensure "cron")
           (directory/ensure "/tmp/test"))
  (array (svc/ensure "cron") (directory/ensure "/tmp/test")))
