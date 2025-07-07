(use ./defaults) ## removed in internal library

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own. Not sure how, yet.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [resource-type]
  (get default-protos resource-type {}))

(defn argcat
  "Joins arguments to make a command"
  [& chunks]
  (string/join (tuple ;chunks) " "))

(defn pathcat
  "Joins tokens to make a path"
  [& chunks]
  (string/join (map |(string/trim $ "/") (tuple "" ;chunks)) "/"))

(defn zfscat
  "Joins tokens to make a ZFS dataset name"
  [& chunks]
  (string/join (map |(string/trim $ "/") (tuple ;chunks)) "/"))

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

(defn qualify-from-path
  "We expect files to be in a directory `files/` at the same level as
  the role file which references those files. This expects a path relative
  to that directory, and returns the fully qualified path, but if it gets
  a fully qualified path, it simply returns it"
  [file-name]
  (if (string/has-prefix? "/" file-name)
    file-name
    (pathcat (dyn :gurp-config-root) "files" file-name)))

(defn resource-id
  [resource-type resource-name resource-spec]
  (string "/" (dyn :role-dyn "NO-ROLE")
          "/" resource-type
          "/" (get (table ;resource-spec) :label
                   (string/replace-all "/" "_" resource-name))))


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

(defmacro host
  "The top-level wrapper used to define a host to be configured"
  [host-name & host-definition]
  ~(upscope
     (setdyn :host-dyn (string ,host-name))
     (defn machine-config
       []
       {:metadata {:name ,host-name}
        :resources (group-by-action-and-type (flatten (tuple ,;host-definition)))})))


(defn group-by-action-and-type
  "Turns an array of resources into a struct of structs, and resolves references."
  [data]

  # We'll assume all structs are resources. That lets us put any other code
  # anywhere we like
  # 
  (def data (clean-data data))
  (def flat-data (mapcat values data))

  (var resolve-reference nil)
  (set resolve-reference
       (fn [val seen]
         (if (keyword? val)
           (let [last-sep (last (string/find-all "/" val))
                 chunks (string/split "/" val last-sep)
                 id (first chunks)
                 field (keyword (last chunks))
                 referenced-struct (find |(= id (get $ :_id)) flat-data)]

             (if (nil? referenced-struct)
               (error (string/format "Referenced resource '%s' not found" id)))

             (if (has-value? seen id)
               (error (string/format "Detected circular reference [%q]" seen)))

             (array/push seen id)

             (def referenced-val (referenced-struct field))

             (if (keyword? referenced-val)
               (resolve-reference referenced-val seen)
               referenced-val))
           val)))

  (defn resolve-resource [res]
    (struct
      ;(mapcat |
               [$ (if (= $ :action) (res $) (resolve-reference (res $) @[]))]
               (keys res))))

  (defn collate [aggr item]
    (each resource-type (keys item)
      (let [resource-raw (item resource-type)
            resource (resolve-resource resource-raw)
            action (resource :action)]
        (when action
          (if-not (aggr action)
            (put aggr action @{}))
          (put aggr action
               (let [curr-map (aggr action)
                     curr-list (get curr-map resource-type @[])]
                 (put curr-map resource-type (array/concat curr-list [resource]))
                 curr-map))))) aggr)

  (var ret (reduce collate @{} data))
  (each k (keys ret) (put ret k (table/to-struct (ret k))))
  (table/to-struct ret))

(defn collect-resources
  "Helper function for the role macro"
  [& resource-structs]
  (flatten (array ;resource-structs)))


(defmacro role
  "Holder for role definitions"
  [role-name & role-definition]
  ~(defn ,role-name
     []
     (def collector @[])
     (setdyn :role-dyn (string ',role-name))
     (collect-resources collector ,;role-definition)))

(defmacro add [& args]
  ~(array/push collector ,;args))

(defmacro section
  "A no-op which might help you write readable definitions"
  [name & body]
  ~(array ,;body))

(defn this
  "A convenient way to reference a resource in the current role"
  [& args]
  (keyword (string/join (tuple "" (this-role) ;args) "/")))

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

  (if-not (= (length find->replace) (length vars))
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

(def resource-ensure-keys
  {:cron {:supported [:user :minute :hour :day-of-month :month-of-year
                      :day-of-week :command]
          :mandatory [:command]}
   :directory {:supported [:owner :mode :group]}
   :file-line {:supported [:line]
               :mandatory [:line]}
   :file {:supported [:owner :mode :group :content :ignore-pattern :from]}
   :misc {:supported [:nfs-domain :enable-smb :scheduler]}
   :smf {:supported [:description :fmri :default-enabled :single-instance
                     :start-method :stop-method :refresh-method :svc-name]
         :mandatory [:description :fmri]}
   :svc {:supported [:state :restarted-by :reloaded-by]
         :mandatory [:state]}
   :svcprop {:supported [:properties]
             :mandatory [:properties]}
   :symlink {:supported [:source]
             :mandatory [:source]}
   :user {:supported [:uid :primary-group :home-dir :shell :gecos :password-hash]
          :mandatory [:uid :primary-group :home-dir :shell :gecos]}
   :zfs {:supported [:properties]}
   :zone {:supported [:properties]}})

(defn- validate-ensure-spec
  [resource-type resource-name resource-spec]
  (def user-keys (filter |(not (= :label $)) (keys (struct ;resource-spec))))
  (def valid-keys (get resource-ensure-keys resource-type {}))
  (def mandatory-keys (get valid-keys :mandatory []))
  (def supported-keys (tuple :label ;(get valid-keys :supported [])))

  (def missing-mandatory (filter |(not (has-value? user-keys $)) mandatory-keys))
  (if-not (empty? missing-mandatory)
    (error (string resource-type " missing required key(s): " (string/join missing-mandatory ", "))))

  (def unrecognised (filter |(not (has-value? supported-keys $)) user-keys))
  (if-not (empty? unrecognised)
    (error (string/format "%s '%s' has unrecognised key(s): %s" resource-type resource-name (string/join unrecognised ", ")))))

(defn- validate-remove-spec
  [resource-type resource-spec])

(defn- ensure-resource
  "Creates a resource struct of the given type and name. The role is picked up
  from a role-specific dynamic binding, to cut down on user boilerplate"
  [resource-type resource-name resource-spec & opts]
  (if-not (has-value? opts :no-validate)
    (validate-ensure-spec resource-type resource-name resource-spec))
  (->>
    (struct/with-proto
      (proto resource-type)
      :_id (resource-id resource-type resource-name resource-spec)
      :role (dyn :role-dyn)
      :name resource-name
      :action :ensure
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
                  :action :remove
                  :role (dyn :role-dyn)
                  :name resource-name
                  (splice resource-spec))))

(defn cron/ensure
  "Given a name and specification, return a cron ensure struct"
  [name & specs]
  (ensure-resource :cron name specs))

(defn cron/remove
  "Given a name and specification, return a cron remove struct"
  [name & specs]
  (remove-resource :cron name specs))

(defn directory/ensure
  "Given a directory name and specification, return a directory ensure struct"
  [name & specs]
  (ensure-resource :directory name specs))

(defn directory/remove
  "Given a directory name and specification, return a directory remove struct"
  [name & specs]
  (remove-resource :directory name specs))

(defn file/ensure
  "Given a file name and specification, return a file ensure struct"
  [name & specs]
  (def result (ensure-resource :file name specs))
  (var resource (struct/to-table (result :file)))

  (if-let [from-path (resource :from)]
    (do
      (set (resource :from) (qualify-from-path from-path))
      {:file (table/to-struct resource)})
    result))

(defn file/remove
  "Given a file name and specification, return a file remove struct"
  [name & specs]
  (remove-resource :file name specs))

(defn file-line/ensure
  "Given a file name and a line pattern, make sure the file contains the line"
  [name & specs]
  (ensure-resource :file-line name specs))

(defn file-line/remove
  "Given a file name and a line pattern, make sure the file does not contain the line"
  [name & specs]
  (remove-resource :file-line name specs))

(defn gem/ensure
  "Given a a gem name, return a gem ensure struct"
  [name & specs]
  (ensure-resource :gem name specs))

(defn gem/remove
  "Given a gem name, return a gem remove struct"
  [name & specs]
  (remove-resource :gem name specs))

(defn misc/ensure
  "Sets miscellaneous system properties"
  [& specs]
  (ensure-resource :misc "GENERIC" specs))

(defn pkg/ensure
  "Given a a pkg name, return a pkg ensure struct. In OmniOS, the
  pkg version is effectively part of the name, so there are no parameters"
  [name & specs]
  (ensure-resource :pkg name specs))

(defn pkg/remove
  "Given a pkg name, return a pkg remove struct"
  [name & specs]
  (remove-resource :pkg name specs))

(defn smf/ensure
  "Given a name and a manifest description, return an smf ensure struct"
  [name & specs]
  (def res (ensure-resource :smf name specs))

  # Protos don't nest, so we need to do a little bit of work on the sub-structs
  (var result @{:svc-name name})
  (loop [[k v] :pairs (res :smf)]
    (put result k
         (if (struct? v) (table/to-struct (merge (proto k) v)) v)))
  {:smf (table/to-struct result)})

(defn svc/ensure
  "Given a name and state, return a svc ensure struct"
  [name & specs]
  (ensure-resource :svc name specs))

(defn svcprop/ensure
  "Given a name and state, return a svcprop ensure struct"
  [name & specs]
  (def spec-table (table ;specs))
  (var new-properties
    (map (fn [v]
           (match (type v)
             :keyword v
             :number {:type "integer" :value v}
             :boolean {:type "boolean" :value v}
             _ {:type "astring" :value v})) (flatten (pairs (spec-table :properties)))))
  (ensure-resource :svcprop name (tuple :properties (struct ;new-properties)) :no-validate))


(defn svcprop/remove
  "Given a name and state, return a svcprop remove struct"
  [name & specs]
  (remove-resource :svcprop name specs))

(defn symlink/ensure
  "Given a symlink name and specification, return a symlink ensure struct"
  [name & specs]
  (ensure-resource :symlink name specs))

(defn symlink/remove
  "Given a symlink name and specification, return a symlink remove struct"
  [name & specs]
  (remove-resource :symlink name specs))

(defn user/ensure
  "Given a user name and specification, return a user ensure struct"
  [name & specs]
  (ensure-resource :user name specs))

(defn user/remove
  "Given a user name and specification, return a user remove struct"
  [name & specs]
  (remove-resource :user name specs))

(defn zfs/ensure
  "Given a zfs dataset name and specification, return a zfs ensure struct"
  [name & specs]
  (ensure-resource :zfs name specs :no-validate))

(defn zfs/remove
  "Given a zfs dataset name and specification, return a zfs remove struct"
  [name & specs]
  (remove-resource :zfs name specs))

(defn zone/ensure
  "Given a zone name and specification, return a zone ensure struct"
  [name & specs]
  (ensure-resource :zonename specs :no-validate))

(defn zone/remove
  "Given a zone name and specification, return a zone remove struct"
  [name & specs]
  (remove-resource :zone name specs))
