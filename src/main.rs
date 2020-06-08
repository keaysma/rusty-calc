//use std::io::{self, Write};
use std::collections::HashMap;
use rand::Rng;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

struct FunDef<'a>{
	variables : &'a[String],
	standard_values : &'a[f32],
	program : &'a[String],
}

struct Memory<'a>{
	variables: HashMap<String, f32>,
	functions: HashMap<String, FunDef<'a>>,
}

struct Response<T>{
	status: u8,
	value: T
}

impl<T> Response<T>{
	fn ok(value: T) -> Response<T>{
		Response{
			status: 0x00,
			value: value,
		}
	}
	
	fn err(status: u8) -> Response<f32>{
		Response{
			status: status,
			value: 0.0,
		}
	}

	/*fn reject(&self, value: T) -> Response<T>{
		Response{
			status: 0xFF,
			value: value
		}
	}*/

	fn reject() -> Response<f32>{
		Response{
			status: 0xFF,
			value: 0.0,
		}
	}

	fn dummy() -> Response<f32>{
		Response{
			status: 0x00,
			value: 0.0,
		}
	}
}

const QUOTES:&[&str] = &[
	"Made with love!", 
	"57% coffee",
];

//This will parse equations, making necessary calls to parse_input to resolve special inputs
const _OPERATORS : &[&str] = &[
	"^", 
	"*", 
	"/",
	"%", 
	"+", 
	"-"
];

const OPERATORS : &[&str] = &[
	"-", 
	"+", 
	"%",
	"/", 
	"*", 
	"^"
];

fn main() {
	let mut vars : HashMap<String, f32> = HashMap::new();
	let mut funs : HashMap<String, FunDef> = HashMap::new();
	let mut memory = Memory{ variables: vars, functions: funs };
    
	println!("Michaelator v0");
	print_rand_quote();
		
	let mut inline = Editor::<()>::new();
	if inline.load_history("history.txt").is_err() {
		println!("[No history found]");
	}
	
	loop{
		//Handle Input
		let input = inline.readline("> ");
		match input {
			Ok(x) => {
				inline.add_history_entry(x.as_str());
				let eq : String = String::from(x);
				let res : Response<f32> = parse_input(&eq, &mut memory);
				match res.status {
					0x00 => {
						println!("{}", res.value);
					},
					0x01 => println!("error"),
					0x02 => println!("NaN"),
					0x07 => continue,
					0x08 => println!("ok"),
					0xFF => {
						break;
					}
					_ => {
						println!("Unexpected status code encountered!");
						break;
					}
				}
			},
			Err(ReadlineError::Interrupted) => continue,
			Err(ReadlineError::Eof) => break,
			Err(_) => break,
		}
	}
	inline.save_history("history.txt").unwrap();
	println!("Goodbye.");
}

//This interperates the input in as many was as necessary to determine what it is
//The status code here symbolizes that the input is of the correct type, or otherwise
//The value can be, depending on the type, a value to return to the user, or a status code to return to the user
//0xFF denotes bad input for the function
fn parse_input(input: &String, mem: &mut Memory) -> Response<f32>{
	if input == ""{
		return Response::<f32>::err(0x07);
	}

	let res = parse_eq(input, mem);
	if res.status == 0x00{
		mem.variables.insert(String::from("_"), res.value);
		return res;
	}

	let res = parse_def(input, mem);
	if res.status == 0x00{
		return res;
	}

	let res = parse_std_fn(input);
	if res.status != 0xFF{
		//todo return function result where 0 is
		return Response{ status: res.value, value: 0.0 };
	}
	
	Response::<f32>::err(0x01)
}

fn parse_numeric(input: &String, mem: &mut Memory) -> Response<f32>{
	let value : Response<f32> = match input.trim().parse() {
		Ok(value) => {
			Response::ok(value) //Response{ status: 0x00, value: value }
		},
		Err(_) => {
			//(0xFF, 0.0)
			resolve_var(input, mem)
		},
	};

	value
}

fn resolve_var(input: &String, mem: &mut Memory) -> Response<f32>{
	return match mem.variables.get(input) {
		Some(x) => Response::ok(*x),
		_ => Response{ status: 0xFF, value: 0.0 },
	};
}

fn parse_eq(input: &String, mem: &mut Memory) -> Response<f32>{
	if !is_balanced_eq(input){
		return Response::<f32>::err(0x01);
	}
	let objs : (u8, Vec<(u8, String)>) = find_clauses_but_better(input);
	if objs.0 != 0x00 {
		return Response{ status: objs.0, value: 0.0 };
	}
	resolve_eq(&objs.1, mem)
}

fn resolve_eq(input: &Vec<(u8, String)>, mem: &mut Memory) -> Response<f32>{
	if input.len() == 1 {
		//println!("[input type  = {}]", input[0].0);
		//println!("[input value  = {}]", input[0].1);

		match input[0].0 {
			0x00 => return parse_numeric(&input[0].1, mem),
			0x01 => return parse_eq(&input[0].1, mem),
			0x03 => return resolve_fn(&input[0].1, mem),
			_ => return Response{ status: 0xFF, value: 0.0 },
		}
	}
	for op in OPERATORS{
		//println!("looking for {}", op);
		for (i, obj) in input.iter().enumerate().rev(){
			if obj.1 == *op{
				let l = resolve_eq(&input[..i].to_vec(), mem);
				if l.status != 0x00 {
					return Response{ status: 0x03, value: 0.0 };
				}
				let r = resolve_eq(&input[i+1..].to_vec(), mem);
				if r.status != 0x00 {
					return Response{ status: 0x04, value: 0.0 };
				}
				//println!("\n match on {}", op);
				match op {
					&"^" => {
						let value: f32 = l.value.powf(r.value);
						return Response::ok(value);
					},
					&"*" => {
						let value: f32 = l.value * r.value;
						return Response::ok(value);
					},
					&"/" => {
						let value: f32 = l.value / r.value;
						return Response::ok(value);
					},
					&"%" => {
						let value: f32 = l.value % r.value;
						return Response::ok(value);
					}
					&"+" => {
						let value: f32 = l.value + r.value;
						return Response::ok(value);
					},
					&"-" => {
						let value: f32 = l.value - r.value;
						return Response::ok(value);
					},
					_ => continue,
				}
			}
			
		}
	}

	Response{ status: 0xFF, value: 0.0 }
}

fn resolve_fn(input: &String, mem: &mut Memory) -> Response<f32>{
	return match mem.functions.get(input){
		Some(_) => {
			Response::ok(0.0)
		},
		_ => Response::<f32>::reject(),
	}
}

fn parse_def(input: &String, mem: &mut Memory) -> Response<f32>{
	if !input.contains("="){
		return Response::<f32>::reject();
	}
	let parts : Vec<&str> = input.split("=").collect();
	if parts.len() > 2 {
		return Response::<f32>::reject();
	}
	let var : Response<String> = parse_var(String::from(parts[0]));
	if var.status != 0x00 {
		return Response{ status: var.status, value: 0.0 };
	}
	let res : Response<f32> = parse_eq(&String::from(parts[1]), mem);
	if res.status != 0x00 {
		return res;
	}
	mem.variables.insert(var.value, res.value);
	Response::ok(res.value)
}

const VAR_RESTRICTIONS : &[&str] = &["(", ")", "{", "}", "'", "\"", "+", "=", "^", "*", "&", "%", "$", "#", "@", "!", "~", "`", " "];

//Will validate and trim variables for you
fn parse_var(input: String) -> Response<String>{
	//Does not begin with a number
	let var = input.trim();
	let numeric_test = match input.chars().nth(0) {
		Some('0') | Some('1') | Some('2') | Some('3') | Some ('4') | Some('5') | Some('6') | Some('7') | Some('8') | Some('9') => 0xFF,
		_ => 0x00
	};
	if numeric_test == 0xFF {
		return Response{ status: 0xFF, value: input };
	}

	//Does not contain special characters: ( ) { } ' " + = ^ * & % $ # @ ! ~ ` space
	for r in VAR_RESTRICTIONS{
		if var.contains(r){
			return Response{ status: 0xFF, value: input };
		}
	}
	Response::ok(String::from(var))
}

//For pre-built functions
fn parse_std_fn(input: &String) -> Response<u8>{
	if input == "exit"{
		return Response::ok(0xFF);
	}

	if input == "continue"{
		return Response::ok(0x07);
	}

	if input == "ok"{
		return Response::ok(0x08);
	}

	Response{ status: 0xFF, value: 0x0 }
}

//Will validate that there are the correct number of opening-to-closing parenthesis
fn is_balanced_eq(input: &String) -> bool{
	let mut lv:i32 = 0;
	for char in input.chars(){
		match char {
			'(' => lv+=1,
			')' => {
				if lv == 0 {
					return false;
				}
				lv-=1;
			},
			_ => continue,
		}
	}
	if lv != 0{
		return false;
	}
	true
}

fn find_clauses_but_better (input: &String) -> (u8, Vec<(u8, String)>) {
	let mut res = Vec::new();
	let mut index = 0;
	let mut level : i64 = -1;
	let mut term_record : bool = false;
	let mut term_start = 0;
	let mut term_type = 0xFF;

	for char in input.chars() {
		match char {
			'(' => {
				if level == -1 {
					if term_record == true {
						let term = &input[term_start..index];
						res.push((term_type, String::from(term)));
					}

					term_record = true;
					term_start = index;
					term_type = 0x01;
				}

				level += 1;
			},
			')' => {
				level -= 1;

				if level == -1 {
					let term = &input[term_start+1..index];
					res.push((term_type, String::from(term)));

					term_record = false;
					term_start = 0;
					term_type = 0xFF;
				}
			},
			_ => {
				if level == -1 {
					match char {
						'^'|'*'|'/'|'%'|'+' => {
							if term_record {
								let term = &input[term_start..index];
								res.push((term_type, String::from(term)));
							}
							
							let term = &input[index..index+1];
							res.push((0x02, String::from(term)));

							term_record = false;
							term_start = 0;
							term_type = 0x00;
						},
						'-' => {
							if term_record == false {
								//Numeric
								term_record = true;
								term_start = index;
								term_type = 0x00;
							}else{
								//Equation
								let term = &input[term_start..index];
								res.push((term_type, String::from(term)));

								let term = &input[index..index+1];
								res.push((0x02, String::from(term)));

								term_record = false;
								term_start = 0;
								term_type = 0xFF;
							}
						},
						_ => {
							if term_record == false {
								term_record = true;
								term_start = index;
								term_type = 0x00;
							}
						}
					}
				}
			}
		}

		index += 1;
	}

	if term_record == true {
		let term = &input[term_start..index];
		res.push((term_type, String::from(term)));

		//term_record = false;
		//term_start = 0;
		//term_type = 0x00;
	}

	(0x00, res)
}

fn print_rand_quote(){
	//let quotes = List::new();

	let rand_int = rand::thread_rng().gen_range(0, 2);
	println!("{}", QUOTES[rand_int]);
}
