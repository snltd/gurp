(defmacro host [name & body]
  ~(defn machine-config []
     (var roles (array))
     (each role (get (table ,;body) :roles)
       (var sym (symbol (string role "/role")))
       (var fn-entry (get (curenv) sym))
       (when (nil? fn-entry)
         (error (string "No such role: " sym)))
       (var fn-to-call
         (if (table? fn-entry)
           (get fn-entry :value)
           fn-entry))
       (when (not (function? fn-to-call))
         (error (string "Role is not callable: " sym)))
       (array/push roles (fn-to-call)))
     (table/to-struct (table :metadata { :name ,name }
            :vars (get (table ,;body) :vars)
            :resources
            (table/to-struct
            (apply helpers/merge-roles roles))))))

(defmacro role [name & body]
  ~(defn ,name []
     (table ,;body)))

(defmacro ensure [name & opts]
  ~(merge (table ,;opts)
          {:name ,name
           :action :ensure}))

(defmacro remove [name & opts]
  ~(merge (table ,;opts)
          {:name ,name
           :action :remove}))

(defn merge-roles [& roles]
  (var result (table))
  (each role roles
    (each k (keys role)
      (var items (get result k @[]))
      (put result k (array/concat items (get role k)))))
  result)
