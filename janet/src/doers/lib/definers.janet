(import ../../dsl :only [qualify-from-path])
(import ../../defs :only [client-api-version ip-protocols])
(import ./validators :only [validate-spec])

# A user specifies a resource by calling a function such as (file/ensure ...)
# This function reads the user's config, and passes it to (enrich-and-validate),
# along with the doer/action's spec. (enrich-and-validate) pads the config
# with any default values, then compares it with the given spec. If the enriched
# config is valid, the top-level function uses (spec->resource) to turn it into
# a struct which it pushes into the collector.
#
# Said function is created by the (make-fn) macro.
# 
# The spec comes from an `attrs` struct, put in scope by (defdoer). It is
# constructed by the  (build-doer) function and contains:
#  :description - a text description of the doer
#  :ensure - a struct containing:
#     :name-is - string explaining the name. If omitted, the doer may not be
#                used with a name
#     :exactly-one-of - a tuple of properties: the user must supply one
#     :at-most-one-of - like above, but zero or one
#     :notes - a tuple of strings for documentation
#     :props - a struct containing:
#       :mandatory - a struct of mandatory properties name -> spec. Spec is
#                    defined as a struct with:
#         :types - a tuple of permissible types
#         :help - a string used in documentation
#         :default - an optional default value
#       :optional - a struct of optional properties. Like :mandatory but
#                   cannot have a defualt
# 
(defn- server-url
  [server-name from-path]
  (string "http://" server-name "/" defs/client-api-version "/file/" from-path))

(defn- all-props
  [attrs]
  (merge-into @{}
              (get-in attrs [:props :mandatory] {})
              (get-in attrs [:props :optional] {})))

(defn resource-id
  "Safely generate a resource ID"
  [resource-type role name]
  (let [name (if (nil? name) "NO-NAME" (string/replace-all "/" "_" name))]
    (string/format "/%s/%s/%s" role resource-type name)))

(defn spec->resource
  "Decorate a spec struct with everything it needs to become a resource."
  [doer-name resource-name spec-struct]
  (let [role (dyn :role-dyn "NO-ROLE")
        final-id-chunk (get spec-struct :label resource-name)]

    (table/to-struct
      (merge {:_id (resource-id doer-name role final-id-chunk)
              :name resource-name
              :role role}
             spec-struct))))

(defn spec-with-defaults [spec-struct default-prop-values]
  (merge default-prop-values spec-struct))

(defn extract-defaults
  [attrs]
  (->> attrs
       (all-props)
       (pairs)
       (map (fn [[k v]] {k (v :default)}))
       (splice)
       (merge-into @{})))

(defn extract-with-attribute
  [attrs attribute]
  (defn group-fn [prop] ((last prop) attribute))
  (let [groups (group-by group-fn (pairs (all-props attrs)))]
    (tabseq [[target props] :pairs groups] target (map |(first $) props))))

(defn- expand-from-value
  "Used by any resource that takes a :from. e.g. file and system-cert. Give it
  a spec table, and it will turn relative :from paths into fully qualified paths
  or server references, depending on the current execution environment. Returns
  a modified spec-table."
  [spec-table from-path]
  (if-let [server-name (dyn :server-name)]
    (do # Point at the server
      (set (spec-table :from) nil)
      (set (spec-table :from-url) (server-url server-name from-path))
      (set (spec-table :url-is-server) true))
    (let [url-or-qualified-path
          (if (string/find "://" from-path)
            from-path
            (dsl/qualify-from-path from-path))]
      (set (spec-table :from) url-or-qualified-path)))
  spec-table)

(defn safe-table
  [& raw-spec]
  (defn group-fn [f]
    (and (tuple? f) (= 2 (length f)) (table? (last f)) ((last f) :from-helper)))

  (def groups (group-by group-fn raw-spec))
  (def ret (table ;(groups false)))

  (each helper-output (get groups true [])
    (let [[key val] helper-output
          unique? (val :unique)]
      (set [val :unique] nil)
      (set [val :from-helper] nil)
      (if unique?
        (set (ret key) val)
        (let [existing (get ret key @[])]
          (set (ret key) (array/concat existing val))))))
  ret)

(defn enrich-and-validate
  [resource-name action-attrs & raw-spec]
  (var spec-table (safe-table ;raw-spec))

  (when-let [defaults (extract-defaults action-attrs)]
    (set spec-table (spec-with-defaults spec-table defaults)))

  (when-let [from-path (spec-table :from)]
    (set spec-table (expand-from-value spec-table from-path)))

  (set spec-table (validators/validate-spec action-attrs spec-table))

  (when-let [movers (extract-with-attribute action-attrs :put-in)]
    (loop [[target props] :pairs movers]
      (loop [prop :in props]
        (put-in spec-table [target prop] (spec-table prop))
        (set (spec-table prop) nil))))

  (when-let [transformers (extract-with-attribute action-attrs :transform-with)
             proceed (not (empty? transformers))]
    (loop [[function props] :pairs transformers]
      (loop [prop :in props]
        (when-let [tpl (spec-table prop)]
          (set (spec-table prop) (function tpl))))))

  (when-let [finalise-fn (action-attrs :finalise-with)]
    (set spec-table (finalise-fn spec-table)))

  spec-table)

(defmacro pinpoint-error
  "Wraps an error in a string describing the resource which caused it"
  [action & body]
  (with-syms [$e]
    ~(try
       (do ,;body)
       ([$e]
         (errorf "In %s/%s %s: %s" doer ,action resource-name $e)))))

(defmacro make-fn
  [action doer-name &opt no-name]
  ~(defn ,(symbol action)
     ,(if no-name
        '[& spec]
        '[resource-name & spec])
     ,(if no-name '(def resource-name (labelise spec)))
     (pinpoint-error
       ,action
       (->> ;spec
            (enrich-and-validate resource-name
                                 (attrs ,action)
                                 ,;(if-not no-name '[:name resource-name] []))
            (spec->resource ,doer-name resource-name)
            (collector/push ,(keyword action) ,doer-name)))))

(defmacro make-helper-fn
  [helper-name &named no-name prefix-name-with-prop]
  (with-syms [$attrs]
    (def $attrs (symbol (string "attrs-" helper-name)))
    ~(defn ,(symbol helper-name)
       ,(if no-name
          '[& spec]
          '[resource-name & spec])
       ,(if no-name '(def resource-name "NO-NAME"))
       (pinpoint-error
         ,helper-name
         (def output-val
           (enrich-and-validate resource-name
                                (,$attrs :helper)
                                ,;(if-not no-name [:name 'resource-name] [])
                                ;spec
                                :from-helper true
                                :unique (get ,$attrs :unique false)))

         (def output-key (if-let [prop-name ,prefix-name-with-prop
                                  prop-val (output-val prop-name)]
                           (keyword prop-val "-" ,helper-name)
                           ,(keyword helper-name)))

         [output-key output-val]))))

(defn property
  [name types requirement help & opts]
  {:name (keyword name)
   :section requirement
   :spec (table :types types :help help ;opts)})

(defn protocol-opts []
  (catseq [protocol :in defs/ip-protocols]
    (property (string protocol) [:struct] :optional
              (string/format "key-value pairs of valid %s properties" protocol)
              :put-in :protocols)))

# :handle-option deals with things like :exactly-one-of,
# :handle-prop deals with properties
(defn- collect
  [args handle-option handle-prop]
  (var i 0)
  (while (< i (length args))
    (let [arg (args i)]
      (if (keyword? arg)
        (do (handle-option arg (args (+ i 1))) (+= i 2))
        (do (handle-prop arg) (++ i))))))


# used by defdoer
(defn- handle-action
  [action & args]
  (let [ret @{:props @{:mandatory @{} :optional @{}}}]
    (defn set-option [k v] (put ret k v))
    (defn add-prop [p] (put-in ret [:props (p :section) (p :name)] (p :spec)))

    (collect args set-option add-prop)
    {:target :attrs :kind action :data ret}))

(defn ensure [& args] (handle-action :ensure ;args))
(defn remove [& args] (handle-action :remove ;args))
(defn helper [& args] (handle-action :helper ;args))
(defn notes [& notes] {:target :doer :kind :notes :data (tuple ;notes)})

(defn build-doer [description & args]
  (def doer @{:description description})
  (defn set-option [k v] (put doer k v))
  (defn add-section [s] (put doer (s :kind) (s :data)))
  (collect args set-option add-section)

  (when (has-key? doer :name-is)
    (put-in doer [:ensure :props :mandatory :name] {:types [:string]})
    (put-in doer [:remove :props :mandatory :name] {:types [:string]}))

  doer)

(defmacro defdoer
  [doer-name description & body]
  ~(upscope
     (def doer ,(keyword doer-name))
     (def attrs (,build-doer ,description ,;body))))

(defn build-helper [description & args]
  (def helper @{:description description})
  (defn set-option [k v] (put helper k v))
  (defn add-section [s] (put helper (s :kind) (s :data)))
  (collect args set-option add-section)

  (when (has-key? helper :name-is)
    (put-in helper [:helper :props :mandatory :name] {:types [:string]}))

  helper)

(defmacro defhelper
  [doer-name helper-name description & body]
  ~(upscope
     (def doer ,(keyword doer-name))
     (def ,(symbol (string "description-" helper-name)) ,description)
     (def ,(symbol (string "attrs-" helper-name)) (,build-helper ,description ,;body))))
