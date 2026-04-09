(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.selectors :as selectors]))

(defn root-view [store]
  (let [state @store]
    [:main {:class "ll-app"}
     [:nav {:class "ll-nav"}
      [:button {:type "button"
                :on-click #(swap! store assoc :route (routes/feed-route))}
       "Feed"]
      [:button {:type "button"
                :on-click #(swap! store assoc :route (routes/sync-route))}
       "Sync"]
      [:button {:type "button"
                :on-click #(swap! store assoc :route (routes/diagnostics-route))}
       "Diagnostics"]]
     (case (selectors/current-page state)
       :post (pages/post-page state)
       :sync (pages/sync-page state)
       :diagnostics (pages/diagnostics-page state)
       (pages/feed-page state))]))
