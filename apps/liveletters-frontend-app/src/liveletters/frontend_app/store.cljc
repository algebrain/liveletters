(ns liveletters.frontend-app.store
  #?(:cljs (:require [reagent.core :as r]))
  (:require [liveletters.frontend-api.core :as frontend-api]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.state :as state]))

(defn create-store []
  (#?(:cljs r/atom :clj atom) (state/initial-state)))

(defn navigate! [store route]
  (swap! store assoc :route route))

(defn refresh-home-feed! [adapter store]
  (swap! store assoc :feed (or (frontend-api/get-home-feed adapter) [])))

(defn refresh-sync-status! [adapter store]
  (swap! store assoc :sync-status (frontend-api/get-sync-status adapter)))

(defn refresh-incoming-failures! [adapter store]
  (swap! store assoc :incoming-failures (or (frontend-api/list-incoming-failures adapter) [])))

(defn refresh-event-failures! [adapter store]
  (swap! store assoc :event-failures (or (frontend-api/list-event-failures adapter) [])))

(defn load-post-thread! [adapter store post-id]
  (swap! store assoc
         :thread (frontend-api/get-post-thread adapter {:post-id post-id})
         :route (routes/post-route post-id)))

(defn update-create-post-form! [store values]
  (swap! store update :create-post merge values))

(defn submit-create-post! [adapter store]
  (let [{:keys [post-id resource-id author-id created-at body]} (:create-post @store)
        request (frontend-api/create-post-request post-id resource-id author-id created-at body)]
    (frontend-api/create-post adapter request)
    (refresh-home-feed! adapter store)))

(defn init! [adapter store]
  (refresh-home-feed! adapter store)
  (refresh-sync-status! adapter store)
  (refresh-incoming-failures! adapter store)
  (refresh-event-failures! adapter store)
  store)
