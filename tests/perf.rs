#![feature(test)]
extern crate test;
extern crate rselisp;

use test::Bencher;
use rselisp::{Lsp, LispObj};

#[bench]
fn fib(b: &mut Bencher) {
    let mut lsp = Lsp::new();
    let src = r#"
(fset 'fib
  '(lambda (a)
    (if (eq a 1)
	1
      (if (eq a 2)
	  2
	(+ (fib (- a 1)) (fib (- a 2)))))))

(fib 20)
"#.to_owned();
    let ast = lsp.read(&src).unwrap();

    b.iter(|| assert_eq!(Ok(LispObj::Int(10946)), lsp.eval(&ast)));
}

#[bench]
fn cons(b: &mut Bencher) {
    let mut lsp = Lsp::new();
    let src = r#"
(fset 'repeat
  '(lambda (a c)
    (if (eq c 0)
	a
      (cons a (repeat a (- c 1))))))

(fset 'add1
  '(lambda (l)
    (if (listp l)
	(cons (+ 1 (car l)) (add1 (cdr l)))
      l)))

(add1 (repeat 1 100))
"#.to_owned();
    let ast = lsp.read(&src).unwrap();

    b.iter(|| lsp.eval(&ast));
}
