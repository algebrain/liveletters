(ns liveletters.frontend-api.tauri
  (:require [clojure.string :as str]
            [cljs.core :refer [js->clj clj->js]]
            [liveletters.frontend-api.core :as core]
            ["@tauri-apps/api/core" :refer [invoke]]
            ["@tauri-apps/api/event" :refer [emit listen]]))

(defn- kebab->snake-key [value]
  (-> value name (str/replace "-" "_")))

(declare encode-payload)

(defn- encode-value [value]
  (cond
    (map? value) (encode-payload value)
    (vector? value) (mapv encode-value value)
    (seq? value) (mapv encode-value value)
    :else value))

(defn encode-payload [payload]
  (when payload
    (into {}
          (map (fn [[key value]]
                 [(kebab->snake-key key) (encode-value value)]))
          payload)))

(def request-command-names
  #{"create_post" "save_settings" "log_frontend_error"})

(defn command-args [command payload]
  (if (contains? request-command-names command)
    {:request payload}
    payload))

(defn- decode-payload [payload]
  (js->clj payload :keywordize-keys true))

(defn normalize-invoke-error [error]
  (let [decoded (cond
                  (string? error)
                  {:code "invoke_error"
                   :message error
                   :details nil}

                  (instance? js/Error error)
                  {:code "invoke_error"
                   :message (or (.-message error) (str error))
                   :details (some-> error .-stack str)}

                  :else
                  (let [value (decode-payload error)]
                    (cond
                      (map? value)
                      (merge {:code "invoke_error"} value)

                      (string? value)
                      {:code "invoke_error"
                       :message value
                       :details nil}

                      :else
                      {:code "invoke_error"
                       :message (str error)
                       :details nil})))]
    (core/normalize-error decoded)))

(defn tauri-adapter []
  {:invoke-command
   (fn [command payload on-success on-error]
     (-> (invoke command (clj->js (encode-payload (command-args command payload))))
         (.then (fn [response]
                  (on-success (decode-payload response))))
         (.catch (fn [error]
                   (on-error (normalize-invoke-error error))))))
   :emit-event
   (fn [event-name payload]
     (emit event-name (clj->js (encode-payload payload))))
   :subscribe-event
   (fn [event-name handler]
     (-> (listen event-name
                 (fn [event]
                   (handler (decode-payload (.-payload event)))))
         (.catch (fn [_error] nil))))})
