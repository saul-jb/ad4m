use std::io::Read;

use scryer_prolog::machine::Machine;
use scryer_prolog::machine::streams::*;

fn string_2_stream(string: String, machine: &mut Machine) -> Stream {
    let machine_st = machine.prelude_view_and_machine_st().1;
    Stream::from_owned_string(string, &mut machine_st.arena)
}

fn print_output(machine: &mut Machine) {
    let output_bytes: Vec<_> = machine.user_output.bytes().map(|b| b.unwrap()).collect();
    let output_string = String::from_utf8(output_bytes).unwrap();
    println!("{}", output_string);
}

pub fn run() {
    std::thread::spawn(|| {
        
        let mut machine = Machine::with_test_streams();
        
        let facts = String::from("triple(\"a\", \"p1\", \"b\").");
        //let facts = String::from("a.\n\n");
        println!("Loading facts: {}", facts);
        let fact_stream = string_2_stream(facts, &mut machine);
        machine.load_file(String::from("facts.pl").as_str(), fact_stream);
        print_output(&mut machine);
        println!("Facts loaded");
        
        
        //let query = String::from("triple(A,B,C), write(\"A = \"), write(A), nl, write(\"B = \"), write(B), write(\"C = \"), write(C), nl ; write(\"no triple matched\").\n");
        let query = String::from("triple(A,B,C).\n");
        //let query = String::from("write(\"A = \").");
        //let query = String::from("halt.\n");
        println!("Loading query: {}", query);
        let query_stream = string_2_stream(query, &mut machine);
        machine.user_input = query_stream;

        println!("run once");
        machine.run_once();
        println!("run once done");

        print_output(&mut machine);
        println!("print output done");
    });
}   