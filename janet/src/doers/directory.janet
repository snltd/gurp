(use ./definers)
(import ../collector)

(defdoer :directory
  "Create and remove directories. Parents are created like `mkdir -p`, but with
  the owner/group/mode of the gurp process. Removal always removes directory
  contents."
  :name-is "Fully qualified path to directory"

  (ensure
    (property "group" [:string :number] :mandatory
      "The group name or GID the for this directory"
      :default "root")
    (property "mode" [:string] :mandatory
      "Permissions, octal"
      :default "0755")
    (property "owner" [:string :number] :mandatory
      "The username or UID of the user who owns this directory"
      :default "root"))

  (notes
    "Directories are created/removed in the order of a natural sort."
    "Directories are created `mkdir -p` style, but only the mode and owner of
    the specified directory are managed by Gurp. Any directories 'filled in'
    to get to the target path will have their ownership and mode dictated by the
    Gurp process and its umask."
    "If you ensure a directory at a path which already exists but is not a
    directory, Gurp will error."
    "Removing a directory removes all its contents, but does not remove any
    empty ancestors."))

(make-fn :ensure :directory)
(make-fn :remove :directory)
