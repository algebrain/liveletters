(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.selectors :as selectors]))

(defn root-view [store]
  (let [state @store]
    [:main {:class "ll-app"}
     (when (get-in state [:bootstrap :setup-completed?])
       [:nav {:class "ll-nav"}
        [:button {:type "button"
                  :on-click #(swap! store assoc :route (routes/feed-route))}
         "Feed"]
        [:button {:type "button"
                  :on-click #(swap! store assoc :route (routes/sync-route))}
         "Sync"]
        [:button {:type "button"
                  :on-click #(swap! store assoc :route (routes/diagnostics-route))}
         "Diagnostics"]
        [:button {:type "button"
                  :on-click #(swap! store assoc :route (routes/settings-route))}
         "Settings"]])
     (case (selectors/current-page state)
       :initial-setup (pages/initial-setup-page store state)
       :post (pages/post-page store state)
       :sync (pages/sync-page store state)
       :diagnostics (pages/diagnostics-page store state)
       :settings (pages/settings-page store state)
       (pages/feed-page store state))]))
