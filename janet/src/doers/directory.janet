(use ./lib/definers)
(import ./file/checksum :prefix "" :export true)
(import ../collector)

(defdoer :directory
  "Create and remove directories. Parents are created like `mkdir -p`, but with
   the owner/group/mode of the gurp process. Removal always removes directory
   contents."
  :name-is "Fully qualified path to directory"

  (ensure
    :at-most-one-of [:create-from :create-with]
    (property "group" [:string :number] :mandatory
              "The group name or GID the for this directory"
              :default "root")
    (property "mode" [:string] :mandatory
              "Permissions, octal"
              :default "0755")
    (property "owner" [:string :number] :mandatory
              "The username or UID of the user who owns this directory"
              :default "root")
    (property "create-from" [:string] :optional
              "Create the directory by unpacking the given archive")
    (property "create-with" [:string] :optional
              "Create the directoey and unpack the given archive inside it")
    (property "checksum" [:struct] :optional
              "Checksum of archive when using :create-from or create-with. See
               `file/checksum`"))

  (notes
    "Directories are created/removed in the order of a natural sort."
    "Directories are created `mkdir -p` style, but only the mode and owner of
     the specified directory are managed by Gurp. Any directories 'filled in'
     to get to the target path will have their ownership and mode dictated by
     the Gurp process and its umask."
    "If you ensure a directory at a path which already exists but is not a
     directory, Gurp will error."
    "Removing a directory removes all its contents, but does not remove any
     empty ancestors."
    "`:create-from` and `:create-with` only apply when the directory is created.
     They will not modify an existing directory, and Gurp will not automatically
     manage any files created by unpacking the archive."
    "`:create-from` and `:create-with` require GNU `tar`."))

(make-fn :ensure :directory)
(make-fn :remove :directory)
