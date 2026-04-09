(ns liveletters.frontend-app.store
  #?(:cljs (:require [reagent.core :as r]))
  (:require [liveletters.frontend-api.core :as frontend-api]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.state :as state]))

(defn create-store []
  (#?(:cljs r/atom :clj atom) (state/initial-state)))

(defn- current-unix-seconds []
  #?(:cljs (js/Math.floor (/ (.now js/Date) 1000))
     :clj 0))

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

(defn- normalize-create-post-form [form]
  (let [created-at (or (not-empty (str (:created-at form))) "0")
        created-at-value (if (= created-at "0")
                           (current-unix-seconds)
                           (:created-at form))
        timestamp (max 1 created-at-value)]
    (-> form
        (update :body #(or % ""))
        (assoc :created-at timestamp)
        (update :resource-id #(if (seq %) % "blog-1"))
        (update :author-id #(if (seq %) % "alice"))
        (update :post-id #(if (seq %) % (str "post-" timestamp))))))

(defn submit-create-post! [adapter store]
  (let [{:keys [post-id resource-id author-id created-at body] :as form}
        (normalize-create-post-form (:create-post @store))
        _ (swap! store assoc :create-post form)
        request (frontend-api/create-post-request post-id resource-id author-id created-at body)]
    (frontend-api/create-post! adapter request
                               (fn [_response]
                                 (clear-error! store)
                                 (swap! store assoc :create-post (assoc form :post-id "" :created-at 0 :body ""))
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
  (swap! store assoc-in [:runtime :adapter] adapter)
  (subscribe-backend-events! adapter store)
  (refresh-home-feed! adapter store)
  (refresh-sync-status! adapter store)
  (refresh-incoming-failures! adapter store)
  (refresh-event-failures! adapter store)
  store)
