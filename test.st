{
  let { eq; } {a0 a1 => not (a0 `- a1);};
  let { func?; } { a0 => not (number? a0); };
  let { !; } { a0 => a0 0; };
  let { unwrap; } { a0 => if (func? a0) { ! a0; } { a0; }; };
  let { tuple; } { a0 a1 => 
    bind { a0; a1; } { i =>
      if { i; } { a1; } { a0; };
    };
  };
  let { get; } { a0 a1 => * (a0 `+ a1); };
}
