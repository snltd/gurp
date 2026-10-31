(use ./definers)
(use ./validators)
(import ../collector)

(defdoer :group
  "Create and destroy Unix groups."
  :name-is "Group name"
  (ensure
    (property "gid" [:number] :mandatory "The group ID")))

(make-fn :ensure :group)
(make-fn :remove :group)
