(import ../dsl :only [basename fields files-in])
(import ../../test/doers/lib/test-lib :only [example-root])

(defn- code-examples-map-fn [dir]
  (defn map-fn [file] (tuple (dsl/basename file) (slurp file)))
  (struct ;(mapcat map-fn (dsl/files-in dir :file))))

# This def is important because it pulls the examples into the Gurp Janet image
# as a table:  (keyword doer) -> (table (string filename) -> (string example))
(def examples-map
  (let [doer-dirs (dsl/files-in test-lib/example-root :directory)]
    (zipcoll (map keyword (map dsl/basename doer-dirs))
             (map code-examples-map-fn doer-dirs))))

(defn- examples-of-action [doer action]
  (defn filter-fn [[k v]] (when (string/has-prefix? (string action "-") k) v))

  (when-let [examples (examples-map doer)]
    (keep filter-fn (sorted (pairs examples)))))

# Return a tuple of examples for the given doer with the given prefix
(defn of [doer & prefixes]
  (catseq [prefix :in prefixes] (examples-of-action doer prefix)))

