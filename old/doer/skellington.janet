# Every key in the defaults requires a (-now) and (-correct)
# function. We can iterate over the keys, aligning each
# property with the input from (must).

(def -defaults
  "(-correct) will fall back to the properties defined here
  if the user has not defined them"
  @{})

(defn -create
  "Creates a new object if it does not exist"
  [obj & opts])

(defn -destroy
  "Destroys the object"
  [obj & opts])

(defn prop-now
  "Fetches the present state of the object, returning it
  in the same form as is defined when calling (must)"
  [obj])

(defn prop-correct
  "Changes the present state of the object to that defined
  by (must)"
  [ojb have want])

(defn must
  "Defines the attributes the object must have"
  [obj & opts])

(defn must-not
  "Defines an object which must not exist"
  [obj & opts])
