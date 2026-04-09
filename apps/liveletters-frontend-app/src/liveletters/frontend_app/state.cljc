(ns liveletters.frontend-app.state)

(defn initial-state []
  {:route {:page :feed}
   :feed []
   :thread nil
   :sync-status nil
   :incoming-failures []
   :create-post {:post-id ""
                 :resource-id ""
                 :author-id ""
                 :created-at 0
                 :body ""}
   :ui {:loading? false
        :error nil}})
