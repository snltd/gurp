# Functions mostly exposed to the user and also used internally.

(defmacro host
  "The top-level wrapper used to define a host to be configured"
  [host-name & host-definition]
  ~(upscope
     (setdyn :host-dyn (string ,host-name))
     (defn machine-config
       "Calling this function evaluates a Janet machine description, populating
       and finalising the *collector*"
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

(defn argcat
  "Joins arguments to make a command"
  [& chunks]
  (string/join (tuple ;chunks) " "))

(defn pathcat
  "Joins tokens to make a path"
  [& chunks]
  (->
    (map |(string/trim $ "/") (tuple "" ;chunks))
    (string/join "/")))

(defn zfscat
  "Joins tokens to make a ZFS dataset name"
  [& chunks]
  (if (nil? chunks)
    (error "zfscat called with a nil"))
  (->
    (map |(string/trim $ "/") (tuple ;chunks))
    (string/join "/")
    (string/trim "/")))

(defn compact
  "Remove empty elements from an array"
  [vector]
  (filter |(not (empty? $)) vector))

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
          (string "cannot qualify path for "
                  file-name
                  ": gurp-config-root is not set")))
      (pathcat (dyn :gurp-config-root) "files" file-name))))

(defn parent
  "Returns the parent directory of the given path"
  [path]
  (def components
    (peg/match ~{:main (some (choice (capture (some (if-not "/" 1))) 1))} path))

  (array/pop components)
  (string "/" (string/join components "/")))

(defn fields
  "Returns an array of the whitespace-separated elements in a string"
  [str]
  (peg/match ~{:main (some (choice (capture :S+) 1))} str))

(defn labelise
  "Turns tokens into a safe label"
  [& chunks]
  (string/replace-all "/"
                      "_"
                      (string/join (map string (flatten chunks)) "-")))

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

(defn cron-minutes-from-name
  "Given a string (usually hostname) and an interval in minutes, return the
  minutes past the hour at which gurp should run, as a comma-separated string"
  [seed-string interval]

  (if-not (= (% 60 interval) 0)
    (error (string interval " is not a divisor of 60")))

  (def seed (% (apply + (seq [c :in seed-string] c)) interval))
  (string/join (map string (seq [i :range [seed 60 interval]] i)) ","))

(defn- values-as-tuple
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

(defn indoc
  "Removes common leading spaces from multiline strings, adding a newline at the
  end if there isn't one already. Start the first line on a new line."
  [str]
  (if-not (string? str)
    (error (string "indoc: expected a string literal, got " str)))

  (def lines (string/split "\n" str))

  (def leader-to-remove
    (->>
      lines
      (filter |(not (empty? (string/trim $))))
      (map |(peg/find :S $))
      (min-of)
      (string/repeat " ")))

  (def outdented-lines
    (if (empty? leader-to-remove)
      lines
      (map |(string/replace leader-to-remove "" $) lines)))

  (def outdented-block (string/join outdented-lines "\n"))

  (if (string/has-suffix? "\n" outdented-block)
    outdented-block
    (string outdented-block "\n")))

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

(defn run-cmd
  "Returns stdout of the given command, or an error containing stderr"
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
