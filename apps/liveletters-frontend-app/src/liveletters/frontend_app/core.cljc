(ns liveletters.frontend-app.core
  (:require [liveletters.frontend-app.app :as app]
            [liveletters.frontend-app.store :as store]))

(defn module-info []
  {:module :liveletters-frontend-app
   :language :cljc})

(def create-app-state store/create-store)
(def navigate! store/navigate!)
(def refresh-home-feed! store/refresh-home-feed!)
(def refresh-bootstrap-state! store/refresh-bootstrap-state!)
(def refresh-sync-status! store/refresh-sync-status!)
(def refresh-incoming-failures! store/refresh-incoming-failures!)
(def refresh-event-failures! store/refresh-event-failures!)
(def load-settings! store/load-settings!)
(def load-post-thread! store/load-post-thread!)
(def update-create-post-form! store/update-create-post-form!)
(def update-settings-form! store/update-settings-form!)
(def submit-create-post! store/submit-create-post!)
(def submit-settings! store/submit-settings!)
(def root-view app/root-view)

(defn init! [adapter app-state]
  (store/init! adapter app-state))
