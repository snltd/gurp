(use ./lib)
(import ../collector)

(def doer :directory)
(def description
  "Create and remove directories. Parents are created like mkdir -p, but with
   the owner/group/mode of the gurp process. Removal always removes directory
   contents.")
(def name-is "Fully qualified path to directory")
(def mandatory-props-ensure
  {:group {:types [:string :number]
           :help "The group name or GID of the for this directory"}
   :mode {:types [:string]
          :help "Permissions, written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:owner "root"
   :mode "0755"
   :group "root"})
(def defaults-remove {})

(defn ensure
  "Given a directory path and spec, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a directory path, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Directories are created/removed in the order of a natural sort."
   "Directories are created 'mkdir -p' style, but only the mode and owner of
    the specified directory are managed by Gurp. Any directories 'filled in'
    to get to the target path will have their ownership and mode dictated by the
    Gurp process and its umask."
   "Removing a directory removes all its contents, but does not remove any
    empty ancestors."])
