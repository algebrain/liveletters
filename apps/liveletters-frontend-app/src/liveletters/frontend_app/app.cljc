(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.selectors :as selectors]))

(defn root-view [store]
  (let [state @store]
    [:main {:class "ll-app"}
     [:nav {:class "ll-nav"}
      [:button {:type "button"} "Feed"]
      [:button {:type "button"} "Sync"]
      [:button {:type "button"} "Diagnostics"]]
     (case (selectors/current-page state)
       :post (pages/post-page state)
       :sync (pages/sync-page state)
       :diagnostics (pages/diagnostics-page state)
       (pages/feed-page state))]))
