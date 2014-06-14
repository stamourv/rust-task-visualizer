#lang racket

(require future-visualizer future-visualizer/trace)

(define log-regexp
  "^Message { timestamp: ([0-9]+), task_id: ([0-9]+), thread_id: ([0-9]+), creator: ([0-9]+), desc: (.+) }")

(define index 0)
(define (next-index!) (begin0 index (set! index (add1 index))))

;; parse-event : string? -> (listof indexed-future-event?)
;; maybe generate 0, 1 or more future events from one rust event
(define (parse-event s)
  (match (regexp-match log-regexp s)
    [(list _ timestamp-string task-id-string thread-id-string creator-string desc)
     (define timestamp (/ (string->number timestamp-string)
                          ;; to be on the same scale as future timestamps
                          1000000.0))
     (define task-id   (string->number task-id-string))
     (define thread-id (string->number thread-id-string))
     (define creator   (string->number creator-string))
     (and timestamp task-id thread-id
          (build-indexed-events timestamp task-id thread-id creator desc))]
    ;; probably some random program output, ignore
    [_ '()]))

;; build-indexed-events : integer? integer? integer? integer? string?
;;                         -> (listof indexed-future-event?)
;; the index serves to disambiguate order in case of identical timestamps
;; (struct future-event (future-id proc-id action time prim-name user-data))
(define (build-indexed-events timestamp task-id thread-id creator desc)
  (define kinds (desc->kinds desc))
  (define (make kind)
    (indexed-future-event
     (next-index!)
     (future-event (match kind
                     ['create (match creator [0 #f] [n n])] ; seems to be part of the protocol
                     [_ task-id])
                   thread-id
                   kind
                   timestamp
                   #f
                   (match kind ; user-data field
                     ['create task-id]
                     ;; none of the other messages we produce need user data
                     [_ #f]))))
  (map make kinds))

;; desc->kinds : string? -> (listof symbol?)
;; returns a list of the kinds of future events that should be emitted
;; based on the rust event observed
(define (desc->kinds desc)
  (match desc
    ["spawn"        '(create start-work)]
    ["before-spawn" '(sync)]
    ["after-spawn"  '(start-work)]
    ["death"        '(complete)]
    ["yield"        '(sync)]
    ["done-yield"   '(start-work)]
    ["maybe-yield"  '(sync)]
    ["deschedule"   '(sync)]
    ["wakeup"       '(start-work)]))



(module+ main
  (define events (append-map parse-event (port->lines)))
  ;; (for-each displayln events)
  (show-visualizer #:timeline events)
  )
