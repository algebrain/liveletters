(ns liveletters.frontend-app.routes)

(defn initial-setup-route []
  {:page :initial-setup})

(defn feed-route []
  {:page :feed})

(defn post-route [post-id]
  {:page :post
   :post-id post-id})

(defn sync-route []
  {:page :sync})

(defn diagnostics-route []
  {:page :diagnostics})

(defn settings-route []
  {:page :settings})
