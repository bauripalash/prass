use std::collections::HashMap;

use pras::compiler::code::make_ins;
use pras::compiler::code::Opcode::*;
use pras::compiler::code::Instructions;

fn check_ins(a : Vec<Vec<u8>> , b : &str){
    let mut x : Instructions = Instructions::new();
    for i in &a{
        x.add_ins(i.to_owned())
    } 

    assert_eq!(x.to_string() , b.to_string())
}

#[test]
fn test_ins() {
   let test_cases = HashMap::from([
        (vec![make_ins(OpAdd, &vec![])] , "0000 OpAdd\n"),

        (vec![make_ins(OpAdd, &vec![]) , make_ins(OpGetLocal, &vec![1]) ] , "0000 OpAdd\n0001 OpGetLocal 1\n"),
   ]);

   for (k,v) in test_cases{
        check_ins(k , v)
   }


}
