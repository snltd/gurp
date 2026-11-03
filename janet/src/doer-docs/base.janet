(import ./lib)
(import ./examples :only [of])

(def Base
  @{:new (fn [class doer &opt helper]
           (let [attrs (if helper (symbol :attrs- helper) :attrs)]
             (table/setproto @{:doer doer
                               :helper helper
                               :info (lib/lookup doer attrs)} class)))

    :title (fn [self & parts] ((self :header-1) (lib/a/b ;parts)))

    :props (fn [self action-info]
             (lib/joint
               (:properties self :mandatory
                            (lib/props-without-name action-info :mandatory))
               (:properties self "Optional"
                            (lib/props-without-name action-info :optional))))

    :action (fn [self action]
              (if-let [doer (self :doer)
                       action-info (get-in self [:info action])]
                (lib/joint ((self :header-2) (lib/a/b doer action))
                           (:props self action-info))
                (string/format "There is no %s/%s action." doer action)))

    :notes (fn [self]
             (when-let [notes (get-in self [:info :notes])]
               (lib/joint ((self :header-2) "Notes")
                          ;(map (self :note-formatter) notes))))

    :examples (fn [self examples]
                (lib/joint ((self :header-2) "Examples")
                           ;(map (self :example-formatter) examples)))

    :render-doer (fn [self]
                   (let [doer (self :doer)]
                     (lib/joint
                       (:title self doer)
                       (:description self (get-in self [:info :description]))
                       (:name self (get-in self [:info :name-is]))
                       (:action self :ensure)
                       (:action self :remove)
                       (:notes self)
                       (:examples self (examples/of doer :ensure :remove)))))

    :render-helper (fn [self]
                     (let [doer (self :doer)
                           helper (self :helper)]
                       (lib/joint
                         (:title self doer helper)
                         (:description self (get-in self [:info :description]))
                         (:name self (get-in self [:info :name-is]))
                         (:action self :helper)
                         (:notes self)
                         (:examples self (examples/of doer helper)))))})
