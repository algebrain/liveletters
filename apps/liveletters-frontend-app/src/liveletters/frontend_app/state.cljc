(ns liveletters.frontend-app.state)

(defn initial-state []
  {:route {:page :feed}
   :feed []
   :thread nil
   :sync-status nil
   :incoming-failures []
   :event-failures []
   :create-post {:post-id ""
                 :resource-id "blog-1"
                 :author-id "alice"
                 :created-at 0
                 :body ""}
   :runtime {:adapter nil}
   :ui {:loading? false
        :error nil}})
