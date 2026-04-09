(ns liveletters.frontend-app.runtime
  (:require [reagent.dom.client :as reagent-dom]
            [liveletters.frontend-api.core :as frontend-api]
            [liveletters.frontend-api.tauri :as tauri-api]
            [liveletters.frontend-app.core :as core]))

(defonce app-state (core/create-app-state))
(defonce root* (atom nil))
(defonce adapter* (atom nil))

(defn- log-runtime-error! [kind message stack source location]
  (when-let [adapter @adapter*]
    (-> (frontend-api/emit-event!
         adapter
         "frontend-error"
         {:kind kind
          :message (or message "unknown frontend error")
          :stack stack
          :source source
          :location location})
        (.catch (fn [_] nil)))))

(defn- install-error-hooks! []
  (set! (.-onerror js/window)
        (fn [message source lineno colno error]
          (let [stack (some-> error .-stack str)
                location (when (and lineno colno)
                           (str lineno ":" colno))]
            (log-runtime-error! "window.onerror" (str message) stack (some-> source str) location))
          false))

  (.addEventListener js/window "unhandledrejection"
                     (fn [event]
                       (let [reason (.-reason event)
                             message (if (string? reason)
                                       reason
                                       (str reason))
                             stack (when (and reason (.-stack reason))
                                     (str (.-stack reason)))]
                         (log-runtime-error! "unhandledrejection" message stack nil nil)))))

(defn mount! []
  (let [container (.getElementById js/document "app")]
    (when container
      (let [root (or @root* (reagent-dom/create-root container))]
        (reset! root* root)
        (reagent-dom/render root [core/root-view app-state])))))

(defn init! []
  (let [adapter (tauri-api/tauri-adapter)]
    (reset! adapter* adapter)
    (install-error-hooks!)
    (core/init! adapter app-state))
  (mount!))
