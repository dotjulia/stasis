{
  let { eq; } {a0 a1 => not (a0 `- a1);};
  let { func?; } { a0 => not (number? a0); };
  let { unwrap; } { a0 => if (func? a0) { ! a0; } { a0; }; };
  let { tuple; } { a0 a1 => 
    bind { a0; a1; } { i =>
      if { i; } { a1; } { a0; };
    };
  };
  let { get; } { a0 a1 => * (a0 `+ a1); };
  let { while; } { __cond __func => 
    if (__cond 0) {
      __func 0;
      while __cond __func;
    } { 0; };
  };
  let { tonumber; } { a0 =>
    let { result; } (alloc 1);
    let { i; } (alloc 1);
    let { multiplicator; } (alloc 1);
    multiplicator `= 1;
    i `= (len a0 `- 1);
    while { i `* `eq (- 0 1) `not; } {
      result `= (result `* `+ (get a0 (* i) `- 48 `mul (* multiplicator)));
      multiplicator `= (multiplicator `* `mul 10);
      i `= (i `* `- 1);
    };
    (* result);
  };
  let { map; } { vec func =>
    let { new_vec; } (vec `len `alloc);
    let { i; } (alloc 1);
    while { i `* `eq (len vec) `not; } {
      (new_vec `+ (* i)) `= (func (get vec (* i)));
      i `= (i `* `+ 1);
    };
    new_vec;
  };
  let { foreach; } { vec func =>
    let { i; } (alloc 1);
    while { i `* `eq (len vec) `not; } {
      func (get vec (* i));
      i `= (i `* `+ 1);
    };
  };
  let {printlist;} { __l => 
    foreach __l { e => print e; };
  };
  let {list;} { __f => __f `inspect `map { str => str `tonumber; }; };
  let {reduce;} {__red_vec __red_func =>
    let {sum;} (alloc 1);
    foreach __red_vec { __red_el =>
      __red_func __red_el sum;
    };
    * sum;
  };
  let {!;} { command => command `inspect `* `exec `printstr; };
  let {sumlen;} { __sl_vec => __sl_vec `reduce { e acc => acc `= ((* acc) `+ (len e)); }; };
  let { append; } { __aarr __el => 
    let { __anew; } (__aarr `len `+ 1 `alloc);
    foreach __aarr { __aelo => (__anew `+ (* i)) `= __aelo; };
    (__anew `+ (len __aarr)) `= __el;
    __anew;
  };
  let { [; } { __first[] => 
    if (number? __first[]) {
      let {__mem[];} (alloc 1);
      __mem[] `= __first[];
      let { __[]inner; } { __el[] => 
        if (number? __el[]) {
          let {__mem[];} (__mem[] `append __el[]);
          bind { __mem[]; __[]inner; } __[]inner;
        } {
          __mem[];
        };
      };
      bind { __mem[]; __[]inner; } __[]inner;
    } {
      0;
    };
  };
  let { ]; } { 0; };
}
