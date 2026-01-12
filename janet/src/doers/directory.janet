(use ./lib)
(import ../collector)

(def doer :directory)
(def description
  "Create and remove directories. Parents are created like mkdir -p, but with
   the owner/group/mode of the gurp process. Removal always removes directory
   contents.")
(def name-is "Fully qualified path to directory")
(def mandatory-ensure-props
  {:group {:types [:string :number]
           :help "The group name or GID of the for this directory"}
   :mode {:types [:string]
          :help "Permissions, written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}})
(def optional-ensure-props {})
(def mandatory-remove-props {})
(def optional-remove-props {})
(def default-prop-values
  {:owner "root"
   :mode "0755"
   :group "root"})

(defn ensure
  "Given a directory name and specification, return a directory ensure struct"
  [name & spec]
  (collector/push :ensure doer (make-resource)))

(defn remove
  "Given a directory name and specification, return a directory remove struct"
  [name & spec]
  (collector/push :remove doer (make-resource)))
