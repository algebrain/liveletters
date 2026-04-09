(ns liveletters.frontend-app.selectors)

(defn current-route [state]
  (:route state))

(defn current-page [state]
  (get-in state [:route :page]))

(defn feed [state]
  (:feed state))

(defn thread [state]
  (:thread state))

(defn sync-status [state]
  (:sync-status state))

(defn incoming-failures [state]
  (:incoming-failures state))

(defn event-failures [state]
  (:event-failures state))

(defn create-post-form [state]
  (:create-post state))
