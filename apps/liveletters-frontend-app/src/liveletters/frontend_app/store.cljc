(ns liveletters.frontend-app.store
  #?(:cljs (:require [reagent.core :as r]))
  (:require [liveletters.frontend-api.core :as frontend-api]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.state :as state]))

(defn create-store []
  (#?(:cljs r/atom :clj atom) (state/initial-state)))

(defn navigate! [store route]
  (swap! store assoc :route route))

(defn- set-error! [store error]
  (swap! store assoc-in [:ui :error] error))

(defn- clear-error! [store]
  (swap! store assoc-in [:ui :error] nil))

(defn refresh-home-feed! [adapter store]
  (frontend-api/get-home-feed! adapter
                               (fn [feed]
                                 (clear-error! store)
                                 (swap! store assoc :feed (or feed [])))
                               #(set-error! store %)))

(defn refresh-sync-status! [adapter store]
  (frontend-api/get-sync-status! adapter
                                 (fn [sync-status]
                                   (clear-error! store)
                                   (swap! store assoc :sync-status sync-status))
                                 #(set-error! store %)))

(defn refresh-incoming-failures! [adapter store]
  (frontend-api/list-incoming-failures! adapter
                                        (fn [failures]
                                          (clear-error! store)
                                          (swap! store assoc :incoming-failures (or failures [])))
                                        #(set-error! store %)))

(defn refresh-event-failures! [adapter store]
  (frontend-api/list-event-failures! adapter
                                     (fn [failures]
                                       (clear-error! store)
                                       (swap! store assoc :event-failures (or failures [])))
                                     #(set-error! store %)))

(defn load-post-thread! [adapter store post-id]
  (frontend-api/get-post-thread! adapter
                                 {:post-id post-id}
                                 (fn [thread]
                                   (clear-error! store)
                                   (swap! store assoc
                                          :thread thread
                                          :route (routes/post-route post-id)))
                                 #(set-error! store %)))

(defn update-create-post-form! [store values]
  (swap! store update :create-post merge values))

(defn submit-create-post! [adapter store]
  (let [{:keys [post-id resource-id author-id created-at body]} (:create-post @store)
        request (frontend-api/create-post-request post-id resource-id author-id created-at body)]
    (frontend-api/create-post! adapter request
                               (fn [_response]
                                 (clear-error! store)
                                 (refresh-home-feed! adapter store)
                                 (refresh-sync-status! adapter store))
                               #(set-error! store %))))

(defn subscribe-backend-events! [adapter store]
  (frontend-api/subscribe-feed-updated!
   adapter
   (fn [_event]
     (refresh-home-feed! adapter store)))
  (frontend-api/subscribe-sync-status-changed!
   adapter
   (fn [_event]
     (refresh-sync-status! adapter store)
     (refresh-incoming-failures! adapter store)
     (refresh-event-failures! adapter store))))

(defn init! [adapter store]
  (subscribe-backend-events! adapter store)
  (refresh-home-feed! adapter store)
  (refresh-sync-status! adapter store)
  (refresh-incoming-failures! adapter store)
  (refresh-event-failures! adapter store)
  store)
