# We only need this import for testing. Gurp bundles the defaults file.
(if-not (get (curenv) (symbol :default-protos))
  (use ./defaults))

(defn new-collector
  "Sets en emptyu *collector*. In a function as most tests use it"
  [] @{:ensure @{} :remove @{}})

# Yes, a GLOBAL VARIABLE! It collects all the resources from the host we
# are configuring. :ensure and :remove are tables whose keys are resource
# types and values are arrays of resources
# 
(var *collector* (new-collector))

(def resource-ensure-keys
  "Pretty much the instructions for Gurp. Defines the specs for all resource
  types. Used by (validate-ensure-spec) to validate user input, and by (help-for)
  to display help"

  {:cron
   {:optional
    {:day-of-month ["Day(s) of month on which job runs" :string :number]
     :day-of-week ["Numeric day(s) on  which job runs. 0=Sunday" :string :number]
     :hour ["Hour(s) at which job runs" :string :number]
     :minute ["Minute(s) job runs at. Accepts divisions and ranges" :string :number]
     :month-of-year ["Month(s) in which job runs" :string :number]
     :user ["Username which runs job. Must already exist" :string]}
    :mandatory
    {:command ["Command which runs" :string]}}

   :directory
   {:optional
    {:group ["The group name or GID of the for this directory" :string :number]
     :mode ["Permissions written as a four-digit octal" :string]
     :owner ["The username or UID of the user who owns this directory" :string :number]}}

   :file-line
   {:mandatory
    {:line ["The line which must exist" :string]}}

   :file
   {:optional
    {:group ["The group name or GID of the for this file" :string :number]
     :mode ["Permissions written as a four-digit octal" :string]
     :owner ["The username or UID of the user who owns this file" :string :number]
     :content ["Literal content of the file. Must have :content xor :from" :string]
     :from ["Copy content from this file. If relative, looks in ../files" :string]
     :ignore-pattern ["When comparing, ignore lines matching this Rust regex" :string]}}

   :gem
   {:optional
    {:gem-path ["Path to gem executable other than /opt/ooce/bin/gem" :string]
     :source ["Source other than RubyGems. Can contain tokens and usernames" :string]
     :version ["Gem version" :string]}}

   :group
   {:mandatory
    {:gid ["The group ID" :number]}}

   :misc
   {:optional
    {:enable-smb ["Enable SMB sharing for this username" :string]
     :nfs-domain ["NFS domain name" :string]
     :scheduler ["The scheduler class to set via dispamdin" :string]}}

   :publisher
   {:mandatory
    {:uri ["Add a pkg publiser with this URI" :string]}}

   :smf
   {:optional
    {:description ["What the service does" :string]
     :duration ["Use this to specify 'transient' or 'wait' services" :string]
     :properties ["Create/set properties.(:keyword :string|:boolean|:number)" :struct]
     :property-groups ["Create property groups (:string)" :tuple]
     :refresh-method ["See 'smf-method'"]
     :single-instance ["Is this a single-instance service" :boolean]
     :start-method ["See 'smf-method'"]
     :stop-method ["See 'smf-method'"]
     :default-enabled ["Start the service when the manifest installs" :boolean]}
    :mandatory
    {:fmri ["Service FMRI" :string]}}

   :svc
   {:optional
    {:reloaded-by ["Labels of resources whose alteration triggers service restart" :tuple]
     :restarted-by ["Labels of resources whose alteration triggers service restart" :tuple]}
    :mandatory
    {:state ["Desired state of service, e.g. 'online'" :string]}}

   :svcprop
   {:optional
    {:property-groups ["Property groups (:string) to create" :tuple]}
    :mandatory
    {:properties ["Properties to create. (:keyword :string|:boolean|:number)" :struct]}}

   :symlink
   {:mandatory
    {:source ["The file the symlink points to" :string]}}

   :user
   {:optional
    {:other-groups ["Group names (:string) or GIDs (:number) to which user belongs" :tuple]
     :password-hash ["Hash to insert in /etc/shadow" :string]
     :profiles ["List of existing profiles (:string)" :tuple]}
    :mandatory
    {:gecos ["User's name or description" :string]
     :home-dir ["User's home dir" :string]
     :primary-group ["Group name or GID to which user belongs" :string :number]
     :shell ["User's shell" :string]
     :uid ["UID of user" :number]}}

   :zfs
   {:optional
    {:properties ["ZFS properties (:keyword) paired with desired value (:string)" :struct]
     :size ["If specified, creates a ZFS volume of given size (e.g. '10G')" :string]}}

   :zone
   {:optional
    {:attr ["See 'zone-attr'"]
     :autoboot ["Boot the zone on system boot" :string]
     :boot-after-install ["Boot the zone n it is installed" :string]
     :bootstrap-from ["Copy gurp into the zone, and apply the given file, relative to zone root" :string]
     :capped-memory ["Set memory cap. Keys must be :physical and :swap, values are strings like '4G'" :struct]
     :clone-from ["Instead of installing, clone from the given zone, which must exist and be halted" :string]
     :copy-in ["Copy files into the zone. Key (keyword) is src, val is dest, relative to zone root. Unqualified src is assumed to be in ../files/" :struct]
     :datasets ["ZFS datasets (as strings) to be delegated to zone" :tuple]
     :dns ["DNS info. :domain is a string; :nameservers a tuple of strings" :struct]
     :exec-in ["Runs the given commands (:string) in the zone after booting" :tuple]
     :fs ["See 'zone-fs'"]
     :lx-image ["Install zone using this image. See docs for pattern rules" :string]
     :net ["See 'zone-net'"]
     :rctl ["See 'zone-rctl'"]
     :recreate ["1-in-n chance the zone will be destroyed and recreated" :number]
     :zonepath ["Path to zone root" :string]}
    :mandatory
    {:brand ["Zone brand. byhve and illumos are not " :string]}}

   :zone-attr
   {:optional
    {:type ["The type of the value. Gurp will take a pretty good guess though" :string]}
    :mandatory
    {:value ["Attribute value" :string :boolean :number]}}

   :zone-rctl
   {:mandatory
    {:value ["rctl value" :string :number]}}

   :zone-fs
   {:optional
    {:type ["The type of fs mount" :string]}
    :mandatory {:special ["The directory in the global zone" :string]}}})

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own. Not sure how, yet.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [resource-type]
  (get default-protos resource-type {}))

(defn pathcat
  "Joins tokens to make a path"
  [& chunks]
  (->
    (map |(string/trim $ "/") (tuple "" ;chunks))
    (string/join "/")))

(defn- clean-data
  "Removes anything which is not a struct from a list"
  [list]
  (filter |(struct? $) list))

(defn parent
  [path]
  (def components
    (peg/match ~{:main (some (choice (capture (some (if-not "/" 1))) 1))} path))

  (array/pop components)
  (string "/" (string/join components "/")))

(defn- qualify-from-path
  "We expect files to be in a directory `files/` at the same level as
  the role file which references those files. This expects a path relative
  to that directory, and returns the fully qualified path, but if it gets
  a fully qualified path, it simply returns it"
  [file-name]
  (if (string/has-prefix? "/" file-name)
    file-name
    (pathcat (dyn :gurp-config-root) "files" file-name)))

(defn- resource-id
  "Uniformly generate a resource ID"
  [resource-type resource-name resource-spec]
  (string/format "/%s/%s/%s"
                 (dyn :role-dyn "NO-ROLE")
                 resource-type
                 (get (table ;resource-spec) :label
                      (string/replace-all "/" "_" resource-name))))

(defmacro- flat-table
  "Flattens a struct or table, including its keys"
  [table]
  ~(flatten (pairs ,table)))

(defn- expand-svc-property
  "Turns a svcprop value into a struct describing a typed value"
  [value]
  (match (type value)
    :keyword value
    :number {:type "integer" :value value}
    :boolean {:type "boolean" :value value}
    _ {:type "astring" :value value}))

(defmacro expand-resource
  "Group the results of in-resource functions like (zone-fs) into a list under
  a single key. Partitions `modified-specs` items whose keys do or do not match
  the given `key`. The matches ('is' group) are flattened into a single array
  (or, if `:as-struct` is passed, reduced to the first matching struct) and
  stored under `key`. Non-matching items ('is-not' group) are preserved."
  [key &keys {:as-struct as-struct}]
  (with-syms [$is-key $key-list $vals]
    ~(do
       (let [$is-key
             (group-by |(and (struct? $) (deep= @[,key] (keys $))) modified-specs)]

         (if-let [$key-list ($is-key true)]
           (let [$vals (mapcat values $key-list)]
             (set modified-specs
                  (tuple ;(get $is-key false @[]) ,key (if ,as-struct (first $vals) $vals)))))))))

(defn check-unique-ids
  "If there are any duplicate resource IDs, throw an error"
  [resource-list]
  (let [seen (table)]
    (loop [id :in (map |($ :_id) resource-list)]
      (if (has-key? seen id) (error (string "duplicate key: " id))
        (set (seen id) true)))
    true))

(defn- resolve-reference
  "Recursively chase down references. Catches circular and dangling"
  [target flat-resources seen]
  (let [last-sep (last (string/find-all "/" target))
        chunks (string/split "/" target last-sep)
        id (first chunks)
        field (keyword (last chunks))
        referenced-struct (find |(= id (get $ :_id)) flat-resources)]

    (if (nil? referenced-struct)
      (error (string/format "Referenced resource '%s' not found" id)))

    (if (has-value? seen id)
      (error (string/format "Detected circular reference [%q]" seen)))

    (set (seen id) true)
    (def referenced-val (referenced-struct field))

    (if (keyword? referenced-val)
      (resolve-reference referenced-val flat-resources seen)
      referenced-val)))

(defn- resolve-resource-references
  "Update any references in a resource with their final targets"
  [resource flat-resources]
  (loop [[k v] :pairs resource]
    (if (keyword? v)
      (set (resource k) (resolve-reference v flat-resources @{}))))
  resource)

(defn- resolved-list
  "resource-list is a list of resources of the same type"
  [resource-list flat-resources]
  (map
    |(table/to-struct (resolve-resource-references (struct/to-table $)
                                                   flat-resources))
    resource-list))

(defn- finalise-action
  "Check there are no duplicate IDs, and resolve any references"
  [resources]
  (let [flat-resources (mapcat values resources)]
    (var ret @{})

    (loop [[resource-type resource-list] :pairs resources]
      (do
        (check-unique-ids resource-list)
        (set (ret resource-type) (resolved-list resource-list flat-resources))))
    ret))

(defn finalise
  [collector]
  {:ensure (finalise-action (collector :ensure))
   :remove (finalise-action (collector :remove))})

(defn- validate-ensure-spec
  "Checks a resource has the keys it should have, according to the 
  resource-ensure-keys struct"
  [resource-type resource-name resource-spec]
  (let [user-keys (filter |(not (= :label $)) (keys (struct ;resource-spec)))
        valid-keys (get resource-ensure-keys resource-type {})
        mandatory-keys (keys (get valid-keys :mandatory []))
        optional-keys (tuple :label ;(keys (get valid-keys :optional {})))
        missing-mandatory (filter |(not (has-value? user-keys $)) mandatory-keys)]

    (if-not (empty? missing-mandatory)
      (error
        (string/format "%s missing required key(s): %s"
                       resource-type
                       (string/join missing-mandatory ", "))))

    (let [unrecognised (filter
                         |(not (has-value? (tuple/join optional-keys mandatory-keys) $))
                         user-keys)]
      (if-not (empty? unrecognised)
        (error
          (string/format "%s '%s' has unrecognised key(s): %s"
                         resource-type
                         resource-name
                         (string/join unrecognised ", ")))))))

(defn- validate-remove-spec
  [resource-type resource-spec])

(defn- collect
  "Put a resource into the collector"
  [action resource-type resource]
  (let [action-struct (*collector* action)]
    (if-not (has-key? action-struct resource-type)
      (set (action-struct resource-type) (array)))
    (array/concat (action-struct resource-type) (first (values resource)))))

(defn- ensure-resource
  "Creates a resource struct of the given type and name. The role is picked up
  from a role-specific dynamic binding, to cut down on user boilerplate.
  resource-spec is an even-length tuple"
  [resource-type resource-name resource-spec & opts]
  (if-not (has-value? opts :no-validate)
    (try
      (validate-ensure-spec resource-type resource-name resource-spec)
      ([e]
        (error
          (string "Failed to validate user input for " resource-type " '" resource-name "' : " e)))))
  (->>
    (struct/with-proto
      (proto resource-type)
      :_id (resource-id resource-type resource-name resource-spec)
      :role (dyn :role-dyn)
      :name resource-name
      (splice resource-spec))
    (struct/proto-flatten)
    (merge)
    (table/to-struct)
    (struct resource-type)))

(defn- remove-resource
  "Creates a resource struct of the given type and name. The role is picked up
  from a role-specific dynamic binding, to cut down on user boilerplate"
  [resource-type resource-name resource-spec]
  (validate-remove-spec resource-type resource-spec)
  (struct resource-type
          (struct :_id (resource-id resource-type resource-name resource-spec)
                  :role (dyn :role-dyn)
                  :name resource-name
                  (splice resource-spec))))
(defn help-for
  "Returns a multiline string showing keys supported by the given resource"
  [resource]

  (defn- field-width
    "Returns the width of a field wide enough to accomodate the longest keys"
    [keys]
    (+ 2 (max (splice (map length keys)))))

  (defn- format-string
    "Returns a format string used to lay out resource key information"
    [key-field-width]
    (string "  %-" key-field-width "s %-15s %s%s"))

  (defn- default-suffix
    "Returns a string snippet displaying a key's default value, if it has one"
    [default-value]
    (if default-value
      (string/format ". Default '%s'" (string default-value))
      ""))

  (defn- keys-for-resource
    [info]
    (tuple/join (keys (get info :mandatory {})) (keys (get info :optional {}))))

  (defn- keys-of-type
    "Returns an array of lines describing either mandatory or optional keys for
    the given 'info' object taken from the resource-ensure-keys struct"
    [info key-type key-field-width]
    (if-let [key-info (info key-type)]
      (let [ret (array (string key-type " keys"))]
        (loop [[key desc] :pairs key-info]
          (let [default-val (get-in default-protos [(keyword resource) key])]
            (array/push ret
                        (string/format (format-string key-field-width)
                                       key
                                       (string/join (array/slice desc 1) "|")
                                       (first desc)
                                       (default-suffix default-val)))))
        ret)
      (string "No " key-type " keys")))

  (if-let [info (resource-ensure-keys (keyword resource))
           key-field-width (field-width (keys-for-resource info))]
    (string/join
      (array/concat
        @[resource]
        (keys-of-type info :mandatory key-field-width)
        (keys-of-type info :optional key-field-width))
      "\n")
    (string/format "No help for '%s'" resource)))

#---- HELPERS FOR THE USER ---------------------------------------------------

(defmacro host
  "The top-level wrapper used to define a host to be configured"
  [host-name & host-definition]
  ~(upscope
     (setdyn :host-dyn (string ,host-name))
     (defn machine-config
       []
       ,;host-definition
       {:metadata {:name ,host-name}
        :resources (finalise *collector*)})))

(defmacro role
  "Holder for role definitions"
  [role-name & role-definition]
  ~(defn ,role-name
     []
     (setdyn :role-dyn (string ',role-name))
     ,;role-definition))

(defmacro section
  "A no-op which might help you write readable definitions"
  [name & body]
  ~(array ,;body))

(defn this-host
  "Returns the name of the host, which is set by a dyn in the host macro"
  []
  (dyn :host-dyn))

(defn this-host-k
  "Returns the name of the host as a keyword. This is set by a dyn in the host macro"
  []
  (keyword (this-host)))

(defn this-role
  "Returns the name of the role, set by a dyn in the role macro"
  []
  (dyn :role-dyn))

(defn this-role-k
  "Returns the name of the role as a keyword, set by a dyn in the role macro"
  []
  (keyword (this-role)))

(defn this
  "A convenient way to reference a resource in the current role"
  [& args]
  (keyword (string/join (tuple "" (this-role) ;args) "/")))

(defn argcat
  "Joins arguments to make a command"
  [& chunks]
  (string/join (tuple ;chunks) " "))

(defn zfscat
  "Joins tokens to make a ZFS dataset name"
  [& chunks]
  (->
    (map |(string/trim $ "/") (tuple ;chunks))
    (string/join "/")
    (string/trim "/")))

(defn labelise
  "Turns tokens into a safe label"
  [& chunks]
  (string/replace-all "/" "_"
                      (string/join (map string chunks) "-")))

(defn template-out
  "Takes a template with vars in {{ brackets }} and a table of vars to values.
  Returns a string or an error"
  [template vars]

  (def peg
    ~{:main (some (choice :subst 1))
      :subst (capture (* :open :value :close))
      :open (* "{{" (any :s))
      :close (* (any :s) "}}")
      :value (/ (capture (some (if-not (set " \t\r\n\0\f\v}") 1))) ,|(vars (keyword $)))})

  (def find->replace (table ;(reverse (peg/match peg template))))
  (var result template)

  (loop [[str-f str-r] :pairs find->replace]
    (set result (string/replace-all str-f str-r result)))

  (def leftovers (peg/match peg result))

  (if-not (empty? leftovers)
    (error (string "unpopulated fields in template: "
                   (string/join (filter |(not (nil? $)) leftovers) ", "))))

  (def patterns
    (map
      |(keyword (string/trim (peg/replace-all '(set "{} ") "" $)))
      (keys find->replace)))

  (def unused-vars
    (filter |(not (has-value? patterns $)) (keys vars)))

  (if-not (empty? unused-vars)
    (error (string/format "unused vars: expected %s : got %s"
                          (string/join
                            (map |(peg/replace-all '(set "{} \t\r\n\0\f\v") "" $)
                                 (keys find->replace)) ", ")
                          (string/join (keys vars) ", "))))
  result)

(defmacro indoc
  "Removes common leading spaces from multiline strings"
  [name str]
  ~(def ,name (string
                (if-not (string? ,str)
                  (error "indoc: expected a string literal"))
                (->
                  (->>
                    ,str
                    (string/split "\n")
                    (filter |(not (empty? (string/trim $))))
                    (map |(peg/find :S $))
                    (min-of)
                    (string/repeat " ")
                    (string "\n"))
                  (string/split (string "\n" ,str))
                  (string/join "\n")
                  (string/triml)))))


(defn fields
  "Returns an array of the whitespace-separated elements in a string"
  [str]
  (peg/match ~{:main (some (choice (capture :S+) 1))} str))

(defn run-cmd
  "Returns stdout of the given command, or an error containting stderr"
  [cmd]
  (def proc (os/spawn (fields cmd) :p {:out :pipe :err :pipe}))
  (:wait proc)
  (def stdout (:read (proc :out) :all))
  (if (nil? stdout)
    (error (string/trim (:read (proc :err) :all)))
    (string/trim stdout)))

(defn hostname
  "Returns the name of the current host"
  []
  (run-cmd "/bin/uname -n"))

#---- RESOURCE ENSURE AND REMOVE ---------------------------------------------

(defn cron/ensure
  "Given a name and specification, return a cron ensure struct"
  [name & specs]
  (collect :ensure :cron (ensure-resource :cron name specs)))

(defn cron/remove
  "Given a name and specification, return a cron remove struct"
  [name & specs]
  (collect :remove :cron (remove-resource :cron name specs)))

(defn directory/ensure
  "Given a directory name and specification, return a directory ensure struct"
  [name & specs]
  (collect :ensure :directory (ensure-resource :directory name specs)))

(defn directory/remove
  "Given a directory name and specification, return a directory remove struct"
  [name & specs]
  (collect :remove :directory (remove-resource :directory name specs)))

(defn file/ensure
  "Given a file name and specification, return a file ensure struct"
  [name & specs]
  (let [result (ensure-resource :file name specs)
        resource (struct/to-table (result :file))]

    (def final-resource
      (if-let [from-path (resource :from)]
        (do
          (set (resource :from) (qualify-from-path from-path))
          {:file (table/to-struct resource)})
        result))

    (collect :ensure :file final-resource)))

(defn file/remove
  "Given a file name and specification, return a file remove struct"
  [name & specs]
  (collect :remove :file (remove-resource :file name specs)))

(defn file-line/ensure
  "Given a file name and a line pattern, make sure the file contains the line"
  [name & specs]
  (collect :ensure :file-line (ensure-resource :file-line name specs)))

(defn file-line/remove
  "Given a file name and a line pattern, make sure the file does not contain the line"
  [name & specs]
  (collect :remove :file-line (remove-resource :file-line name specs)))

(defn gem/ensure
  "Given a a gem name, return a gem ensure struct"
  [name & specs]
  (collect :ensure :gem (ensure-resource :gem name specs)))

(defn gem/remove
  "Given a gem name, return a gem remove struct"
  [name & specs]
  (collect :remove :gem (remove-resource :gem name specs)))

(defn group/ensure
  "Given a group name and specification, return a group ensure struct"
  [name & specs]
  (collect :ensure :group (ensure-resource :group name specs)))

(defn group/remove
  "Given a group name and specification, return a group remove struct"
  [name & specs]
  (collect :remove :group (remove-resource :group name specs)))

(defn misc/ensure
  "Sets miscellaneous system properties"
  [& specs]
  (collect :ensure :misc (ensure-resource :misc (labelise ;specs) specs)))

(defn pkg/ensure
  "Given a a pkg name, return a pkg ensure struct. In OmniOS, the
  pkg version is effectively part of the name, so there are no parameters"
  [name & specs]
  (collect :ensure :pkg (ensure-resource :pkg name specs)))

(defn pkg/remove
  "Given a pkg name, return a pkg remove struct"
  [name & specs]
  (collect :remove :pkg (remove-resource :pkg name specs)))

(defn pkgin/ensure
  "Given a a pkgin name, return a pkgin ensure struct."
  [name & specs]
  (collect :ensure :pkgin (ensure-resource :pkgin name specs)))

(defn pkgin/remove
  "Given a pkgin name, return a pkgin remove struct"
  [name & specs]
  (collect :remove :pkgin (remove-resource :pkgin name specs)))

(defn publisher/ensure
  "Given a a publisher name, return a publisher ensure struct"
  [name & specs]
  (collect :ensure :publisher (ensure-resource :publisher name specs)))

(defn publisher/remove
  "Given a publisher name, return a publisher remove struct"
  [name & specs]
  (collect :remove :publisher (remove-resource :publisher name specs)))

(defn smf/ensure
  "Given a name and a manifest description, return an smf ensure struct"
  [name & specs]
  (var modified-specs specs)
  (expand-resource :start-method :as-struct true)
  (expand-resource :stop-method :as-struct true)
  (expand-resource :restart-method :as-struct true)
  (expand-resource :refresh-method :as-struct true)

  (let [spec-table (table (splice modified-specs))]
    (when (has-key? spec-table :properties)
      (def expanded-properties
        (struct
          ;(map expand-svc-property (flat-table (spec-table :properties)))))
      (set (spec-table :properties) expanded-properties))

    (collect :ensure :smf (ensure-resource :smf name (flat-table spec-table)))))

(def smf-context-keys [:user :group :privileges :environment])

(defn smf-method
  "A convenience function to help produce an SMF exec_method, with a context"
  [action & specs]
  (let [actions ["start" "stop" "refresh" "reload"]]
    (if-not (has-value? actions action)
      (error (string "action must be one of " (string/join actions ", ")))))

  (let [spec-table (table (splice specs))
        context-keys (filter |(has-value? smf-context-keys $) (keys spec-table))
        context @{}]

    (if-not (has-key? spec-table :exec)
      (error "smf-method requires an :exec"))

    (table/setproto spec-table (struct/to-table (proto :smf-method)))

    (each k context-keys
      (set (context k)
           (if (= k :privileges) (string/join (spec-table k) ",") (spec-table k)))
      (set (spec-table k) nil))

    (if-not (empty? context)
      (set (spec-table :context) (table/to-struct context)))

    (struct
      (keyword (string action "-method"))
      (table/proto-flatten spec-table))))

(defn svc/ensure
  "Given a name and state, return a svc ensure struct"
  [name & specs]
  (collect :ensure :svc (ensure-resource :svc name specs)))

(defn svcprop/ensure
  "Given a name and state, return a svcprop ensure struct"
  [name & specs]
  (let [result (ensure-resource :svcprop name specs)]
    (var resource (struct/to-table (result :svcprop)))

    (var new-properties
      (map expand-svc-property (flat-table (resource :properties))))

    (set (resource :properties) (struct ;new-properties))
    (collect :ensure :svcprop (struct :svcprop (table/to-struct resource)))))

(defn svcprop/remove
  "Given a name and state, return a svcprop remove struct"
  [name & specs]
  (collect :remove :svcprop (remove-resource :svcprop name specs)))

(defn symlink/ensure
  "Given a symlink name and specification, return a symlink ensure struct"
  [name & specs]
  (collect :ensure :symlink (ensure-resource :symlink name specs)))

(defn symlink/remove
  "Given a symlink name and specification, return a symlink remove struct"
  [name & specs]
  (collect :remove :symlink (remove-resource :symlink name specs)))

(defn user/ensure
  "Given a user name and specification, return a user ensure struct"
  [name & specs]
  (collect :ensure :user (ensure-resource :user name specs)))

(defn user/remove
  "Given a user name and specification, return a user remove struct"
  [name & specs]
  (collect :remove :user (remove-resource :user name specs)))

(defn zfs/ensure
  "Given a zfs dataset name and specification, return a zfs ensure struct"
  [name & specs]
  (collect :ensure :zfs (ensure-resource :zfs name specs)))

(defn zfs/remove
  "Given a zfs dataset name and specification, return a zfs remove struct"
  [name & specs]
  (collect :remove :zfs (remove-resource :zfs name specs)))

(defn zone/ensure
  "Given a zone name and specification, return a zone ensure struct"
  [name & specs]
  (var modified-specs specs)
  (expand-resource :net)
  (expand-resource :attr)
  (expand-resource :fs)
  (expand-resource :rctl)
  (let [result (ensure-resource :zone name modified-specs)
        resource (struct/to-table (result :zone))]

    (if-let [copy-resource (resource :copy-in)]
      (set (resource :copy-in)
           (table/to-struct
             (zipcoll (map qualify-from-path (keys copy-resource))
                      (values copy-resource)))))

    (if-not (has-key? resource :zonepath)
      (set (resource :zonepath) (pathcat "/zones" name)))

    (collect :ensure :zone {:zone (table/to-struct resource)})))

(defn zone/remove
  "Given a zone name and specification, return a zone remove struct"
  [name & specs]
  (collect :remove :zone (remove-resource :zone name specs)))

(defn zone-network
  "Given specs, return a zone network struct. This is embedded in a zone/ensure"
  [physical & specs]
  (struct :net
          (struct/proto-flatten
            (struct/with-proto (proto :zone-network)
                               :physical physical ;specs))))

(defn zone-fs
  "Given specs, return a zone fs struct. This is embedded in a zone/ensure"
  [mountpoint & specs]
  (struct :fs
          (struct/proto-flatten
            (struct/with-proto (proto :zone-fs)
                               :dir mountpoint ;specs))))

(defn zone-attr
  "Given specs, return a zone attr struct. This is embedded in a zone/ensure"
  [name & specs]
  (var spec-table (table :name name ;specs))
  (if-not
    (has-key? spec-table :value)
    (error "zone-attr requires a :value"))

  (if-not (has-key? spec-table :type)
    (set (spec-table :type)
         (match (type (spec-table :value))
           :number "uint"
           :boolean "boolean"
           _ "string")))

  (if (= "string" (spec-table :type))
    (set (spec-table :value) (string (spec-table :value))))

  (struct :attr (table/to-struct spec-table)))

(defn zone-rctl
  "Given specs, return a zone rctl struct. This is embedded in a zone/ensure"
  [name & specs]
  (let [spec-struct (struct/with-proto (proto :zone-rctl) :name name (splice specs))]

    (if-not (has-key? spec-struct :limit)
      (error "zone-rctl requires a :limit"))

    (struct :rctl (struct/proto-flatten spec-struct))))
