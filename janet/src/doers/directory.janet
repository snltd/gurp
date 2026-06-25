(use ./lib)
(import ../collector)

(defdoer :directory
  "Create and remove directories. Parents are created like mkdir -p, but with
  the owner/group/mode of the gurp process. Removal always removes directory
  contents."

  :name-is "Fully qualified path to directory"

  :mandatory-props-ensure
  {:group {:types [:string :number]
           :help "The group name or GID of the for this directory"}
   :mode {:types [:string]
          :help "Permissions, written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}}

  :defaults-ensure
  {:owner "root"
   :mode "0755"
   :group "root"}

  :notes
  ["Directories are created/removed in the order of a natural sort."
   "Directories are created 'mkdir -p' style, but only the mode and owner of
    the specified directory are managed by Gurp. Any directories 'filled in'
    to get to the target path will have their ownership and mode dictated by the
    Gurp process and its umask."
   "If you ensure a directory at a path which already exists but is not a
    directory, Gurp will error"
   "Removing a directory removes all its contents, but does not remove any
    empty ancestors."])

(defensure "directory")
(defremove "directory")
