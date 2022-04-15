;*

    Number passthrough check

*;

(global short eleven (+ 5 6))                                ; (+ 5.0 6.0)
(global short zero (- eleven eleven))                        ; (- eleven eleven), with eleven converted to a real
(global boolean eleven_is_greater_than_zero (= eleven zero)) ; eleven and zero should be shorts here
