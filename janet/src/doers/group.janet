(use ./lib)
(import ../collector)

(defdoer :group
  "Create and destroy Unix groups."
  :name-is "Group name"

  :mandatory-props-ensure
  {:gid {:types [:number]
         :help "The group ID"}})

(defensure "group")
(defremove "group")
