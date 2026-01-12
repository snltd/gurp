(use ./defaults)
(use ./doer-defs)
(use ./formatting)
(use ./helpers)
(import ./commands :prefix "" :export true)
(import ./collector :prefix "" :export true)
(use ./resource-builder)

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own. Not sure how, yet.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [action resource-type]
  (get-in default-protos [action resource-type] {}))

(defn pathcat
  "Joins tokens to make a path"
  [& chunks]
  (->
    (map |(string/trim $ "/") (tuple "" ;chunks))
    (string/join "/")))

(defn qualified-path?
  "Returns true if the argument looks like a fully qualified path"
  [path]
  (string/has-prefix? "/" path))

(defn qualify-from-path
  "We expect files to be in a directory `files/` at the same level as
  the role file which references those files. This expects a path relative
  to that directory, and returns the fully qualified path, but if it gets
  a fully qualified path, it simply returns it"
  [file-name]

  (if (qualified-path? file-name)
    file-name
    (do
      (if (nil? (dyn :gurp-config-root))
        (error
          (string "cannot qualify path for " file-name ": gurp-config-root is not set")))
      (pathcat (dyn :gurp-config-root) "files" file-name))))

(defn- resource-id
  "Uniformly generate a resource ID"
  [resource-type resource-name resource-spec]
  (string/format "/%s/%s/%s"
                 (dyn :role-dyn "NO-ROLE")
                 resource-type
                 (get (table ;resource-spec) :label
                      (string/replace-all "/" "_" resource-name))))

(defn table->tuple
  "Turns a table into a tuple, preserving tuple keys"
  [spec-table]
  (mapcat identity (pairs spec-table)))

(defmacro- table->flat-tuple
  "Completely flattens a struct or table, including its keys"
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
  (try
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
        referenced-val))
    ([e]
      (error
        (string "Failed to resolve reference '" target "'")))))

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
  (if (dyn :destroy-everything-you-touch)
    {:ensure {}
     :remove (finalise-action (collector :ensure))}
    {:ensure (finalise-action (collector :ensure))
     :remove (finalise-action (collector :remove))}))






(defn- has-exactly-one-of?
  "Checks whether spec contains exactly one of the required-keys"
  [required-keys specs]
  (= 1 (length (filter |(has-value? required-keys $) (keys (table ;specs))))))

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


(defn zfscat
  "Joins tokens to make a ZFS dataset name"
  [& chunks]
  (if (nil? chunks)
    (error "zfscat called with a nil"))
  (->
    (map |(string/trim $ "/") (tuple ;chunks))
    (string/join "/")
    (string/trim "/")))

(defn parent
  "Returns the parent directory of the given path"
  [path]
  (let [components
        (peg/match ~{:main (some (choice (capture (some (if-not "/" 1))) 1))} path)]

    (array/pop components)
    (string "/" (string/join components "/"))))

(defn labelise
  "Turns tokens into a safe label"
  [& chunks]
  (string/replace-all "/"
                      "_"
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
    (error (string/format "unused vars: expected %s: got %s"
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
  "Returns the name of the current host, or the name of the calling host if Gurp
  is running as in server mode"
  []
  (if-let [hostname (dyn :client-name)]
    hostname
    (run-cmd "uname -n")))

(defn config-file
  "Returns the actual path of a file in ../files"
  [path]
  (qualify-from-path path))

(defn cloudinit-meta-data
  "Returns a cloudinit meta-data struct for the given hostname"
  [hostname]
  {:instance-id hostname :local-hostname hostname})

(defn cron-minutes-from-name
  "Given a string (usually hostname) and an interval in minutes, return the
  minutes past the hour at which gurp should run, as a comma-separated string"
  [seed-string interval]

  (if-not (= (% 60 interval) 0)
    (error (string interval " is not a divisor of 60")))

  (def seed (% (apply + (seq [c :in seed-string] c)) interval))
  (string/join (map string (seq [i :range [seed 60 interval]] i)) ","))

(defn values-as-tuple
  "Returns a flat array of values, whatever type of values it's given"
  [values]
  (flatten (array values)))

(defn repeated-line-file
  "Produces a string, with a trailing newline, created by mapping the given
  values to a string produced by using each value in the given format string.
  If format-values is an array of arrays, each value of the inner array is used
  in the format string"
  [format-string format-values]
  (->>
    format-values
    (map |(string/format (string format-string "\n") ;(values-as-tuple $)))
    (string/join)))

#---- RESOURCE ENSURE AND REMOVE ---------------------------------------------

(defn apk/ensure
  "Given a a apk name, return an apk ensure struct"
  [name & specs]
  (collect :ensure :apk (make-resource :ensure :apk name specs)))

(defn apk/remove
  "Given a apk name, return an apk remove struct"
  [name & specs]
  (collect :remove :apk (make-resource :remove :apk name specs)))

(defn cron/ensure
  "Given a name and specification, return a cron ensure struct"
  [name & specs]
  (collect :ensure :cron (make-resource :ensure :cron name specs)))

(defn cron/remove
  "Given a name and specification, return a cron remove struct"
  [name & specs]
  (collect :remove :cron (make-resource :remove :cron name specs)))

(defn directory/ensure
  "Given a directory name and specification, return a directory ensure struct"
  [name & specs]
  (collect :ensure :directory (make-resource :ensure :directory name specs)))

(defn directory/remove
  "Given a directory name and specification, return a directory remove struct"
  [name & specs]
  (collect :remove :directory (make-resource :remove :directory name specs)))

(defn file/ensure
  "Given a file name and specification, return a file ensure struct. If Gurp is
   running as a server, changes local file references into HTTP ones."
  [name & specs]
  (let [result (make-resource :ensure :file name specs)
        resource (struct/to-table (result :file))]

    (if-let [from-path (resource :from)]
      (if-let [server-name (dyn :server-name)]
        (do
          (set (resource :from) nil)
          (set (resource :from-url) (string "http://" server-name "/file/" from-path)))
        (let [url-or-qualified-path (if (string/find "://" from-path) from-path (qualify-from-path from-path))]
          (set (resource :from) url-or-qualified-path)
          {:file (table/to-struct resource)})))

    (collect :ensure :file (struct :file (table/to-struct resource)))))

(defn file/remove
  "Given a file name and specification, return a file remove struct"
  [name & specs]
  (collect :remove :file (make-resource :remove :file name specs)))

(defn file-line/ensure
  "Given a file name and a line pattern, make sure the file contains the line"
  [name & specs]
  (collect :ensure :file-line (make-resource :ensure :file-line name specs)))

(defn file-line/remove
  "Given a file name and a line pattern, make sure the file does not contain the line"
  [name & specs]
  (let [match-allowed ["exact" "starts_with" "ends_with" "contains" "matches"]
        apply-to-allowed ["all" "first" "last"]
        spec-struct (struct ;specs)]
    (if-let [match-val (spec-struct :match)]
      (if-not (has-value? match-allowed match-val)
        (error
          (string "match must be one of "
                  (string/join match-allowed ", ") " [Got '" match-val "']"))))

    (if-let [type-val (spec-struct :apply-to)]
      (if-not (has-value? apply-to-allowed type-val)
        (error
          (string "type must be one of " (string/join apply-to-allowed ", "))))))

  (collect :remove :file-line (make-resource :remove :file-line name specs)))

(defn gem/ensure
  "Given a a gem name, return a gem ensure struct"
  [name & specs]
  (collect :ensure :gem (make-resource :ensure :gem name specs)))

(defn gem/remove
  "Given a gem name, return a gem remove struct"
  [name & specs]
  (collect :remove :gem (make-resource :remove :gem name specs)))

(defn group/ensure
  "Given a group name and specification, return a group ensure struct"
  [name & specs]
  (collect :ensure :group (make-resource :ensure :group name specs)))

(defn group/remove
  "Given a group name and specification, return a group remove struct"
  [name & specs]
  (collect :remove :group (make-resource :remove :group name specs)))

(defn ip-address/ensure
  "Given an ip-address name and specification, return an ip-address ensure struct"
  [name & specs]
  (collect :ensure :ip-address (make-resource :ensure :ip-address name specs)))

(defn ip-address/remove
  "Given an ip-address name and specification, return an ip-address remove struct"
  [name & specs]
  (collect :remove :ip-address (make-resource :remove :ip-address name specs)))

(defn ip-interface/ensure
  "Given an interface name and specification, return an ip-interface ensure struct"
  [name & specs]
  (let [protocols @{}
        other-specs @[]]
    (each spec specs
      (if (= (type spec) :struct)
        (merge-into protocols spec)
        (array/concat other-specs spec)))

    (def complete-spec (array/concat other-specs :protocols protocols))

    (collect :ensure :ip-interface
             (make-resource :ensure :ip-interface name complete-spec))))

(defn ip-interface/remove
  "Given an interface name and specification, return an ip-interface remove struct"
  [name & specs]
  (collect :remove :ip-interface (make-resource :remove :ip-interface name specs)))

(defn ip-interface-protocol
  "Given specs, return config for an interface protocol. Key is protocol, values
  are params"
  [protocol & params]
  (struct protocol (struct (splice params))))

(defn ip-properties/ensure
  "Given a protocol and specification, return an ip-properties ensure struct"
  [name & specs]
  (let [spec-struct (struct :properties (struct (splice specs)))]
    (collect :ensure :ip-properties
             (make-resource :ensure :ip-properties name (table->tuple spec-struct)))))


(defn ipnat/ensure
  "Given a name and specification, return an ipnat ensure struct"
  [name & specs]
  (if-not (has-exactly-one-of? [:content :from] specs)
    (error "need exactly one of :content or :from"))

  (collect :ensure :ipnat (make-resource :ensure :ipnat name specs)))

(defn ipnat/remove
  "Given a name, return an ipnat remove struct"
  [name]
  (collect :remove :ipnat (make-resource :remove :ipnat name [])))

(defn misc/ensure
  "Sets miscellaneous system properties"
  [& specs]
  (collect :ensure :misc (make-resource :ensure :misc (labelise ;specs) specs)))

(defn network-flow/ensure
  "Given a flow name and specification, return a network-flow ensure struct"
  [name & specs]
  (collect :ensure :network-flow (make-resource :ensure :network-flow name specs)))

(defn network-flow/remove
  "Given a flow name and specification, return a network-flow remove struct"
  [name & specs]
  (collect :remove :network-flow (make-resource :remove :network-flow name specs)))

(defn pkg/ensure
  "Given a a pkg name, return a pkg ensure struct. In OmniOS, the
  pkg version is effectively part of the name, so there are no parameters"
  [name & specs]
  (collect :ensure :pkg (make-resource :ensure :pkg name specs)))

(defn pkg/remove
  "Given a pkg name, return a pkg remove struct"
  [name & specs]
  (collect :remove :pkg (make-resource :remove :pkg name specs)))

(defn pkgin/ensure
  "Given a a pkgin name, return a pkgin ensure struct."
  [name & specs]
  (collect :ensure :pkgin (make-resource :ensure :pkgin name specs)))

(defn pkgin/remove
  "Given a pkgin name, return a pkgin remove struct"
  [name & specs]
  (collect :remove :pkgin (make-resource :remove :pkgin name specs)))

(defn publisher/ensure
  "Given a a publisher name, return a publisher ensure struct"
  [name & specs]
  (collect :ensure :publisher (make-resource :ensure :publisher name specs)))

(defn publisher/remove
  "Given a publisher name, return a publisher remove struct"
  [name & specs]
  (collect :remove :publisher (make-resource :remove :publisher name specs)))

(defn route/ensure
  "Given a route name and specification, return a route ensure struct"
  [name & specs]
  (let [resource (make-resource :ensure :route name specs)
        resource-keys (keys (resource :route))]

    (if-not (has-exactly-one-of? [:gateway :interface] specs)
      (error "Provide exactly one of :gateway and :interface"))

    (collect :ensure :route resource)))

(defn route/remove
  "Given a route name and specification, return a route remove struct"
  [name & specs]
  (collect :remove :route (make-resource :remove :route name specs)))

(def smf-context-keys [:user :group :privileges :environment])

(defn smf/ensure
  "Given a name and a manifest description, return an SMF service ensure struct"
  [name & specs]
  (var modified-specs specs)
  (expand-resource :dependencies)
  (expand-resource :dependents)
  (expand-resource :start-method :as-struct true)
  (expand-resource :stop-method :as-struct true)
  (expand-resource :restart-method :as-struct true)
  (expand-resource :refresh-method :as-struct true)

  (let [spec-table (table (splice modified-specs))]
    (when (has-key? spec-table :properties)
      (def expanded-properties
        (struct
          ;(map expand-svc-property (table->flat-tuple (spec-table :properties)))))
      (set (spec-table :properties) expanded-properties))

    (collect :ensure :smf (make-resource :ensure :smf name (table->tuple spec-table)))))

(defn smf/remove
  "Given a service name, return an SMF service remove struct"
  [name & specs]
  (collect :remove :smf (make-resource :remove :smf name specs)))

(defn smf-dependency
  "A convenience function to help produce an SMF dependency"
  [name & specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :smf-dependency) :name name)
          (struct/proto-flatten))]

    (validate-spec :ensure :smf-dependency name (table->flat-tuple spec-struct))
    (struct :dependencies spec-struct)))

(defn smf-dependent
  "A convenience function to help produce an SMF dependent"
  [name & specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :smf-dependent) :name name)
          (struct/proto-flatten))]

    (validate-spec :ensure :smf-dependent name (table->flat-tuple spec-struct))
    (struct :dependents spec-struct)))

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

    (table/setproto spec-table (struct/to-table (proto :ensure :smf-method)))

    (each k context-keys
      (set (context k)
           (if (= k :privileges)
             (string/join (spec-table k) ",")
             (spec-table k)))
      (set (spec-table k) nil))

    (if-not (empty? context)
      (set (spec-table :context) (table/to-struct context)))

    (struct
      (keyword (string action "-method"))
      (table/proto-flatten spec-table))))

(defn svc/ensure
  "Given a name and state, return a svc ensure struct"
  [name & specs]
  (let [result (make-resource :ensure :svc name specs)]
    (var resource (struct/to-table (result :svc)))

    (if-let [restarters (resource :restarted-by)]
      (set (resource :restarted-by) (map string restarters)))

    (if-let [reloaders (resource :reloaded-by)]
      (set (resource :reloaded-by) (map string reloaders)))

    (collect :ensure :svc (struct :svc (table/to-struct resource)))))

(defn svcprop/ensure
  "Given a name and state, return a svcprop ensure struct"
  [name & specs]
  (let [result (make-resource :ensure :svcprop name specs)]
    (var resource (struct/to-table (result :svcprop)))

    (var new-properties
      (map expand-svc-property (table->flat-tuple (resource :properties))))

    (set (resource :properties) (struct ;new-properties))
    (collect :ensure :svcprop (struct :svcprop (table/to-struct resource)))))

(defn svcprop/remove
  "Given a name and state, return a svcprop remove struct"
  [name & specs]
  (collect :remove :svcprop (make-resource :remove :svcprop name specs)))

(defn symlink/ensure
  "Given a symlink name and specification, return a symlink ensure struct"
  [name & specs]
  (collect :ensure :symlink (make-resource :ensure :symlink name specs)))

(defn symlink/remove
  "Given a symlink name and specification, return a symlink remove struct"
  [name & specs]
  (collect :remove :symlink (make-resource :remove :symlink name specs)))

(defn user/ensure
  "Given a user name and specification, return a user ensure struct"
  [name & specs]
  (collect :ensure :user (make-resource :ensure :user name specs)))

(defn user/remove
  "Given a user name and specification, return a user remove struct"
  [name & specs]
  (collect :remove :user (make-resource :remove :user name specs)))

(defn vlan/ensure
  "Given a VLAN name and specification, return a vlan ensure struct"
  [name & specs]
  (collect :ensure :vlan (make-resource :ensure :vlan name specs)))

(defn vlan/remove
  "Given a VLAN name and specification, return a vlan remove struct"
  [name & specs]
  (collect :remove :vlan (make-resource :remove :vlan name specs)))

(defn vnic/ensure
  "Given a VNIC name and specification, return a vnic ensure struct"
  [name & specs]
  (collect :ensure :vnic (make-resource :ensure :vnic name specs)))

(defn vnic/remove
  "Given a VNIC name and specification, return a vnic remove struct"
  [name & specs]
  (collect :remove :vnic (make-resource :remove :vnic name specs)))

(defn zfs/ensure
  "Given a zfs dataset name and specification, return a ZFS ensure struct"
  [name & specs]
  (collect :ensure :zfs (make-resource :ensure :zfs name specs)))

(defn zfs/remove
  "Given a zfs dataset name and specification, return a ZFS remove struct"
  [name & specs]
  (collect :remove :zfs (make-resource :remove :zfs name specs)))

(defn zone/ensure
  "Given a zone name and specification, return a zone ensure struct"
  [name & specs]
  (var modified-specs specs)
  (expand-resource :net)
  (expand-resource :attr)
  (expand-resource :fs)
  (expand-resource :rctl)
  (expand-resource :bhyve :as-struct true)
  (expand-resource :bootstrap :as-struct true)
  (let [result (make-resource :ensure :zone name modified-specs)
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
  (collect :remove :zone (make-resource :remove :zone name specs)))

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

(defn zone-bhyve
  "Given specs, return config for a bhyve zone"
  [& specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :zone-bhyve))
          (struct/proto-flatten))]

    (validate-spec :ensure :zone-bhyve nil (table->flat-tuple spec-struct))
    (struct :bhyve spec-struct)))

(defn zone-bootstrap
  "Given specs, return config to bootstrap a zone"
  [& specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :zone-bootstrap))
          (struct/proto-flatten))]

    (validate-spec :ensure :zone-bootstrap nil (table->flat-tuple spec-struct))
    (struct :bootstrap spec-struct)))

(defn zone-fs
  "Given specs, return a zone fs struct. This is embedded in a zone/ensure"
  [mountpoint & specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :zone-fs) :dir mountpoint)
          (struct/proto-flatten))]

    (validate-spec :ensure :zone-fs mountpoint (table->flat-tuple spec-struct))
    (struct :fs spec-struct)))

(defn zone-network
  "Given specs, return a zone network struct. This is embedded in a zone/ensure"
  [physical & specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :zone-network) :physical physical)
          (struct/proto-flatten))]

    (validate-spec :ensure :zone-network physical (table->flat-tuple spec-struct))
    (struct :net spec-struct)))

(defn zone-rctl
  "Given specs, return a zone rctl struct. This is embedded in a zone/ensure"
  [name & specs]
  (let [spec-struct
        (->>
          (splice specs)
          (struct/with-proto (proto :ensure :zone-rctl) :name name)
          (struct/proto-flatten))]

    (validate-spec :ensure :zone-rctl name (table->flat-tuple spec-struct))
    (struct :rctl spec-struct)))
