(ns liveletters.frontend-app.store
  #?(:cljs (:require [reagent.core :as r]))
  (:require [liveletters.frontend-api.core :as frontend-api]
            [clojure.string :as str]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.state :as state]))

#?(:cljs
   (defonce settings-autofill-timers* (atom {})))

(def ^:private username-autofill-delay-ms 1000)

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

(defn refresh-bootstrap-state! [adapter store on-complete]
  (frontend-api/get-bootstrap-state! adapter
                                     (fn [bootstrap]
                                       (clear-error! store)
                                       (swap! store assoc :bootstrap (merge {:checked? true} bootstrap))
                                       (when on-complete
                                         (on-complete bootstrap)))
                                     #(set-error! store %)))

(defn load-settings! [adapter store on-complete]
  (frontend-api/get-settings! adapter
                              (fn [settings]
                                (clear-error! store)
                                (swap! store assoc :settings-form settings)
                                (when on-complete
                                  (on-complete settings)))
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

(defn- blank-string? [value]
  (str/blank? (or value "")))

(defn- apply-linked-username-autofill! [store source-key target-key expected-value]
  (swap! store
         (fn [state]
           (let [current-source (get-in state [:settings-form source-key])
                 current-target (get-in state [:settings-form target-key])]
             (if (and (= current-source expected-value)
                      (not (blank-string? expected-value))
                      (blank-string? current-target))
               (assoc-in state [:settings-form target-key] expected-value)
               state)))))

#?(:cljs
   (defn- schedule-linked-username-autofill! [store source-key target-key source-value]
     (when-let [timer-id (get @settings-autofill-timers* source-key)]
       (js/clearTimeout timer-id))
     (swap! settings-autofill-timers* dissoc source-key)
     (when-not (blank-string? source-value)
       (let [timer-id (js/setTimeout
                       (fn []
                         (swap! settings-autofill-timers* dissoc source-key)
                         (apply-linked-username-autofill! store
                                                          source-key
                                                          target-key
                                                          source-value))
                       username-autofill-delay-ms)]
         (swap! settings-autofill-timers* assoc source-key timer-id)))))

#?(:clj
   (defn- schedule-linked-username-autofill! [_store _source-key _target-key _source-value]
     nil))

(defn- schedule-settings-autofill! [store values]
  (when (contains? values :smtp-username)
    (schedule-linked-username-autofill! store
                                        :smtp-username
                                        :imap-username
                                        (:smtp-username values)))
  (when (contains? values :imap-username)
    (schedule-linked-username-autofill! store
                                        :imap-username
                                        :smtp-username
                                        (:imap-username values))))

(defn update-settings-form! [store values]
  (swap! store update :settings-form merge values)
  (schedule-settings-autofill! store values))

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

(defn- normalize-settings-form [form]
  (-> form
      (update :nickname #(or % ""))
      (update :email-address #(or % ""))
      (update :avatar-url #(or % ""))
      (update :smtp-host #(or % ""))
      (update :smtp-port #(if (string? %)
                            (js/parseInt % 10)
                            %))
      (update :smtp-security #(or % "starttls"))
      (update :smtp-username #(or % ""))
      (update :smtp-password #(or % ""))
      (update :smtp-hello-domain #(or % ""))
      (update :imap-host #(or % ""))
      (update :imap-port #(if (string? %)
                            (js/parseInt % 10)
                            %))
      (update :imap-security #(or % "starttls"))
      (update :imap-username #(or % ""))
      (update :imap-password #(or % ""))
      (update :imap-mailbox #(or % "INBOX"))
      (update :smtp-port #(if (js/isNaN %) 0 %))
      (update :imap-port #(if (js/isNaN %) 0 %))))

(defn submit-settings! [adapter store]
  (let [page (get-in @store [:route :page])
        form (normalize-settings-form (:settings-form @store))
        request (frontend-api/save-settings-request form)]
    (swap! store assoc :settings-form form)
    (frontend-api/save-settings! adapter request
                                 (fn [saved]
                                   (clear-error! store)
                                   (swap! store assoc
                                          :settings-form saved
                                          :bootstrap {:checked? true
                                                      :setup-completed? true})
                                   (if (= page :initial-setup)
                                     (do
                                       (navigate! store (routes/feed-route))
                                       (refresh-home-feed! adapter store)
                                       (refresh-sync-status! adapter store)
                                       (refresh-incoming-failures! adapter store)
                                       (refresh-event-failures! adapter store))
                                     (navigate! store (routes/settings-route))))
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
  (refresh-bootstrap-state!
   adapter
   store
   (fn [bootstrap]
     (load-settings! adapter store nil)
     (if (:setup-completed? bootstrap)
       (do
         (navigate! store (routes/feed-route))
         (refresh-home-feed! adapter store)
         (refresh-sync-status! adapter store)
         (refresh-incoming-failures! adapter store)
         (refresh-event-failures! adapter store))
       (navigate! store (routes/initial-setup-route)))))
  store)
