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

(test-macro
  (expand-resource :attr)
  (do
    (let [$is-key (group-by (short-fn (and (struct? $) (deep= @[:attr] (keys $)))) modified-specs)]
      (if-let [$key-list ($is-key true)]
        (let [$vals (mapcat values $key-list)]
          (set modified-specs (tuple (splice (get $is-key false @[])) :attr (if nil (first $vals) $vals))))))))
