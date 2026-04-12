(ns liveletters.frontend-app.styles
  (:require [lambdaisland.ornament :as o]))

(def global-styles
  "*,*::before,*::after
{box-sizing:border-box}
:root
{--bg-primary:#17212b;--bg-secondary:#0e1621;--bg-tertiary:#1c2733;--bg-hover:#202b36;--text-primary:#f5f5f5;--text-secondary:#708495;--accent:#5eb5f7;--accent-hover:#3aa5f7;--border:#0e1621;--input-bg:#242f3d;--input-border:#2f3b47;--input-focus:#5eb5f7;--modal-overlay:rgba(0,0,0,0.6);--danger:#e05555;--success:#4caf50}
html
{font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",Roboto,Helvetica,Arial,sans-serif;background:var(--bg-primary);color:var(--text-primary)}
body
{margin:0;min-height:100vh;background:var(--bg-primary)}
#app
{min-height:100vh}
button,input
{font:inherit;color:var(--text-primary)}
.ll-nav
{display:flex;flex-wrap:wrap;gap:10px}
.ll-button
{appearance:none;border:none;border-radius:999px;padding:12px 18px;font-size:.98rem;font-weight:600;letter-spacing:.01em;cursor:pointer;transition:transform 160ms ease,box-shadow 160ms ease,background-color 160ms ease}
.ll-button:hover
{transform:translateY(-1px)}
.ll-button:disabled
{cursor:not-allowed;opacity:.55;transform:none}
.ll-button--primary
{background:var(--accent);color:#fff;box-shadow:0 4px 12px rgba(94,181,247,.25)}
.ll-button--primary:hover
{background:var(--accent-hover)}
.ll-button--secondary
{background:rgba(255,255,255,.08);color:var(--text-primary);border:1px solid rgba(255,255,255,.1)}
.ll-section
{background:var(--bg-tertiary);border:1px solid rgba(255,255,255,.06);border-radius:22px}
.ll-section__title
{margin:0;padding:24px 24px 0;font-size:1.5rem;line-height:1.1;letter-spacing:-.02em;color:var(--text-primary)}
.ll-section__body
{display:grid;gap:18px;padding:20px 24px 24px}
.ll-field
{display:grid;gap:8px}
.ll-field__label
{font-size:.82rem;font-weight:700;text-transform:uppercase;letter-spacing:.08em;color:var(--text-secondary)}
.ll-input
{width:100%;min-height:48px;padding:12px 14px;border-radius:12px;border:1px solid var(--input-border);background:var(--input-bg);color:var(--text-primary);outline:none}
.ll-input:focus
{border-color:var(--input-focus);box-shadow:0 0 0 3px rgba(94,181,247,.15)}
select.ll-input
{appearance:none;background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23708495' d='M6 8L1 3h10z'/%3E%3C/svg%3E\");background-repeat:no-repeat;background-position:right 14px center}
select.ll-input option
{background:var(--bg-tertiary);color:var(--text-primary)}
.ll-state
{padding:14px 16px;border-radius:12px;font-size:.96rem}
.ll-state--loading
{background:rgba(255,255,255,.06);color:var(--text-secondary)}
.ll-state--empty
{background:rgba(255,255,255,.04);color:var(--text-secondary)}
.ll-state--error
{background:rgba(224,85,85,.1);border:1px solid rgba(224,85,85,.2);color:var(--danger)}
.ll-feed
{display:grid;gap:12px;padding:0;margin:0;list-style:none}
.ll-feed__item
{padding:16px 18px;border-radius:14px;background:var(--bg-tertiary);border:1px solid rgba(255,255,255,.06);cursor:pointer}
.ll-feed__flag,.ll-post__flag
{display:inline-flex;padding:4px 8px;border-radius:999px;background:rgba(224,85,85,.1);color:var(--danger);font-size:.82rem;font-weight:700}
.ll-thread,.ll-failures,.ll-event-failures
{display:grid;gap:14px;padding:0;margin:0;list-style:none}
.ll-thread__item,.ll-failures__item,.ll-event-failures__item
{padding:16px 18px;border-radius:14px;background:var(--bg-tertiary);border:1px solid rgba(255,255,255,.06)}
.ll-sync--healthy
{border-color:rgba(94,181,247,.25)}
.ll-sync--degraded
{border-color:rgba(255,183,77,.3)}
.ll-sync--failed
{border-color:rgba(224,85,85,.25)}
@media (max-width:720px){
  .ll-section__title{padding:20px 18px 0}
  .ll-section__body{padding:16px 18px 20px}
  .ll-button{width:100%}
}")

(defn ensure-style-namespaces-loaded! []
  (require 'liveletters.frontend-app.theme.core
           'liveletters.frontend-app.theme.shell
           'liveletters.frontend-app.theme.nav
           'liveletters.frontend-app.theme.page
           'liveletters.frontend-app.theme.settings))

(defn write-styles! []
  (ensure-style-namespaces-loaded!)
  (spit "resources/ornament.css"
        (str global-styles
             "\n"
             (o/defined-styles))))

(defn write-styles-hook
  {:shadow.build/stage :flush}
  [build-state & _args]
  (write-styles!)
  build-state)

(defn -main [& _args]
  (write-styles!))
