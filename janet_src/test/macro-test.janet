(use judge)
(use ../lib/gurp)

(test-macro
  (host "example-node" (role "basenode") (role "devtools"))
  (upscope
    (setdyn :host-dyn (string "example-node"))
    (defn machine-config
      []
      {:metadata {:name "example-node"} :resources (group-by-action-and-type (flatten (tuple (role "basenode") (role "devtools"))))})))

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
    (def collector @[])
    (setdyn :role-dyn (string (quote basenode)))
    (collect-resources collector (pkg/ensure "helix") (pkg/remove "go") (file/ensure "/tmp/basenode.txt" :content "some words") (directory/ensure "/tmp/merp" :owner "tester" :group "tester" :mode "0755") (directory/remove "/tmp/junk"))))

(test-macro
  (section "test-section"
           (svc/ensure "cron")
           (directory/ensure "/tmp/test"))
  (array (svc/ensure "cron") (directory/ensure "/tmp/test")))
