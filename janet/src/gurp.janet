(use ./defaults)
(use ./doer-defs)
(use ./formatting)
(use ./helpers)
(import ./commands :prefix "" :export true)
(import ./collector :prefix "" :export true)
(use ./resource-builder)


# 
# Definitions for the doers. Used to display help information, and to sanity-
# check user input.
# 
(def resource-ensure-keys
  "Pretty much the instructions for Gurp. Defines the specs for all resource
  types. Used by (validate-ensure-spec) to validate user input, and by (help-for)
  to display help"

   :smf
   {:description "Create and install a manifest for an SMF service."
    :name "Short name of service. Not used internally"
    :optional
    {:dependencies ["See 'smf-dependency'"]
     :dependents ["See 'smf-dependent'"]
     :description ["What the service does" :string]
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

   :smf-dependency
   {:description "Defines a dependency of an SMF service, inside an smf resource."
    :name "Any convenient name"
    :optional
    {:restart-on ["Policy for restarting this service if dependency restarts" :string]
     :grouping ["Which dependencies are required by this service" :string]
     :type ["Type of dependency" :string]}
    :mandatory
    {:name ["The name of the dependency relationship" :string]
     :fmri ["Dependency FMRI" :string]}}

   :smf-dependent
   {:description "Defines a dependent of an SMF service, inside an smf resource."
    :name "Any convenient name"
    :optional
    {:restart-on ["Policy for restarting this service if dependent restarts" :string]
     :grouping ["Which dependents are required by this service" :string]
     :type ["Type of dependent" :string]}
    :mandatory
    {:name ["The name of the dependent relationship" :string]
     :fmri ["Dependent FMRI" :string]}}

   :svc
   {:description "Manage the state of an existing SMF service."
    :name "Any valid service FMRI"
    :optional
    {:reloaded-by ["Labels of resources whose alteration triggers service restart" :tuple]
     :restarted-by ["Labels of resources whose alteration triggers service restart" :tuple]}
    :mandatory
    {:state ["Desired state of service, e.g. 'online'" :string]}}

   :svcprop
   {:description "Manage properties of an existing SMF service."
    :name "Any valid FMRI of the service whose properties you wish to set"
    :optional
    {:property-groups ["Property groups (:string) to create" :tuple]}
    :mandatory
    {:properties ["Properties to create. (:keyword :string|:boolean|:number)" :struct]}}

   :zfs
   {:description "Create, destroy, and modify properties of ZFS filesystems."
    :name "ZFS dataset name"
    :optional
    {:properties ["ZFS properties (:keyword) paired with desired value (:string)" :struct]
     :size ["If specified, creates a ZFS volume of given size (e.g. '10G')" :string]}
    :mandatory {}}

   :zone
   {:description "Create and destroy zones. Existing zones cannot be modified."
    :name "Zone name"
    :optional
    {:attr ["See 'zone-attr'"]
     :autoboot ["Boot the zone on system boot" :string]
     :bhyve ["See 'zone-bhyve'"]
     :boot-after-install ["Boot the zone n it is installed" :string]
     :bootstrap ["See 'zone-bootstrap'"]
     :bootstrap-from ["Copy gurp into the zone, and apply the given file, relative to zone root" :string]
     :capped-memory ["Set memory cap. Keys must be :physical and :swap, values are strings like '4G'" :struct]
     :clone-from ["Instead of installing, clone from the given zone, which must exist and be halted" :string]
     :copy-in ["Copy files into the zone. Key (keyword) is src, val is dest, relative to zone root. Unqualified src is assumed to be in ../files/" :struct]
     :datasets ["ZFS datasets (as strings) to be delegated to zone" :tuple]
     :dns ["DNS info. :domain is a string; :nameservers a tuple of strings" :struct]
     :exec-in ["Runs the given commands (:string) in the zone after booting" :tuple]
     :final-state ["Put the zone in the given state. Also accepts 'reboot'" :string]
     :fs ["See 'zone-fs'"]
     :ip-type ["IP type: exclusive or shared" :string]
     :hostid ["Force this hostid for the zone" :string]
     :limitpriv ["List of privileges to add to zone" :tuple]
     :lx-image ["Install zone using this image. See docs for pattern rules" :string]
     :net ["See 'zone-network'"]
     :pool ["Resource pool to which zone should belong" :string]
     :rctl ["See 'zone-rctl'"]
     :recreate ["1-in-n chance the zone will be destroyed and recreated" :number]
     :zonepath ["Path to zone root" :string]}
    :mandatory
    {:brand ["Zone brand" :string]}}

   :zone-attr
   {:description "Set attributes on a zone being created by the zone doer."
    :name "Attribute name"
    :optional
    {:type ["The type of the value. Gurp will take a pretty good guess though" :string]}
    :mandatory
    {:value ["Attribute value" :string :boolean :number]}}

   :zone-bhyve
   {:description "Describe a bhyve zone inside a zone resource."
    :name "This resource type does not accept a name"
    :optional
    {:cloudinit-struct ["Generate a Cloudinit file from the given struct. Top level keys map to files, e.g. 'user-data'" :struct]
     :cloudinit-files ["Copy the given files into the Cloudinit image" :tuple]
     :wait-for-boot ["Wait for boot, or detach immediately" :bool]
     :image-url ["URL of remote install image" :string]
     :image-format ["Specify the format of the image pointed to by :image-url" :string]
     :image-path ["Path to install image - must be raw format" :string]}
    :mandatory
    {:ram ["Amount of RAM to allocate: e.g. '3G'" :string]
     :vcpus ["Number of VCPUs to allocate" :number]
     :boot-volume ["ZFS boot volume" :string]}}

   :zone-bootstrap
   {:description "Tells gurp how to bootstrap a newly created zone."
    :name "This resource type does not accept a name"
    :optional
    {:server ["hostname/IP address of server to install from" :string]
     :hostname ["hostname of client being bootstrapped" :string]
     :file ["fully qualified path of file in zone which will be used to bootstrap" :string]}}

   :zone-network
   {:description "Describe network configuration of a zone resource."
    :name "Zone VNIC, which may already exist"
    :optional
    {:global-nic ["Physical NIC on which to create zone VNIC" :string]
     :allowed-address ["IP address, with /netmask" :string]
     :defrouter ["IP address of default router" :string]}
    :mandatory
    {:physical ["Zone VNIC. This is the name of the resource, and is not specified with a key" :string]}}

   :zone-rctl
   {:description "Define a resource control when creating a zone."
    :name "RCTL name"
    :mandatory
    {:priv ["rctl privilege" :string]
     :action ["rctl action" :string]
     :name ["private field managed by Gurp" :string]
     :limit ["rctl limit value" :string :number]}}

   :zone-fs
   {:description "Define a filesystem mapping when creating a zone."
    :name "The mountpoint inside the zone"
    :optional
    {:type ["The type of fs mount" :string]
     :options ["Options with which to mount fs inside zone" :tuple]}
    :mandatory
    {:dir ["Mountpoint in zone. This is the name of the resource, and is not specified with a key" :string]
     :special ["The directory in the global zone" :string]}}})

(def resource-remove-keys
  "Like resource-ensure-keys but for removing resources"
   :smf {:optional {} :mandatory {}}

   :svcprop
   {:optional
    {:property-groups ["Property groups (:string) to create" :tuple]}
    :mandatory
    {:properties ["Properties to create. (:keyword :string|:boolean|:number)" :struct]}}

   :zfs {:optional {} :mandatory {}}
   :zone {:optional {} :mandatory {}}})

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own. Not sure how, yet.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [action resource-type]
  (get-in default-protos [action resource-type] {}))




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


(defn route/ensure
  "Given a route name and specification, return a route ensure struct"
  [name & specs]
  (let [resource (make-resource :ensure :route name specs)
        resource-keys (keys (resource :route))]


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
