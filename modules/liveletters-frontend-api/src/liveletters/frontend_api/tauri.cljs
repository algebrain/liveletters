(ns liveletters.frontend-api.tauri
  (:require [clojure.string :as str]
            [cljs.core :refer [js->clj clj->js]]
            [liveletters.frontend-api.core :as core]
            ["@tauri-apps/api/core" :refer [invoke]]
            ["@tauri-apps/api/event" :refer [emit listen]]))

(defn- kebab->snake-key [value]
  (-> value name (str/replace "-" "_")))

(defn- encode-payload [payload]
  (when payload
    (into {}
          (map (fn [[key value]]
                 [(kebab->snake-key key) value]))
          payload)))

(defn- decode-payload [payload]
  (js->clj payload :keywordize-keys true))

(defn tauri-adapter []
  {:invoke-command
   (fn [command payload on-success on-error]
     (-> (invoke command (clj->js (encode-payload payload)))
         (.then (fn [response]
                  (on-success (decode-payload response))))
         (.catch (fn [error]
                   (on-error (core/normalize-error (decode-payload error)))))))
   :emit-event
   (fn [event-name payload]
     (emit event-name (clj->js (encode-payload payload))))
   :subscribe-event
   (fn [event-name handler]
     (-> (listen event-name
                 (fn [event]
                   (handler (decode-payload (.-payload event)))))
         (.catch (fn [_error] nil))))})
