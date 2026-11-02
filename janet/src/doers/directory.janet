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
          :help "Permissions, octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}}

  :optional-props-ensure
  {:create-from {:types [:hash]
                 :help "If the directory does not exist, create it by unpacking
                 a tar[.gz] file. It is assumed that the archive contains the
                 directory itself. Keys are `:file` if the source is a tar[.gz]
                 file; or `:url` if the source is a URL, with either case
                 accepting an optional SHA256 `:checksum`."}}

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

(defn ensure
  "Given a directory path and spec, put an ensure struct in the collector. If
  Gurp is running as a remote client, changes local file references into HTTP
  ones."
  [name & spec]
  (let [user-table (struct/to-table (make-spec-struct ;spec))
        spec-table (expand-from-value user-table)
        all-specs (spec-with-defaults defaults-ensure spec-table)
        safe-specs (pinpoint-error
                     :ensure
                     (checked-spec all-specs
                                   mandatory-props-ensure
                                   optional-props-ensure))]

    (collector/push :ensure doer (spec->resource doer name safe-specs))))

(defremove "directory")
