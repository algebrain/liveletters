(ns liveletters.ui-model.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.ui-model.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-ui-model
          :language :cljc}
         (core/module-info))))
