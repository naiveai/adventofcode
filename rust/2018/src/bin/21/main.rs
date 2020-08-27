#![feature(fn_traits)]

use anyhow::anyhow;
use clap::{App, Arg};
use itertools::Itertools;
use std::fmt;
use std::fs;
use std::ops;

pub fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("2018-21")
        .arg(Arg::from_usage("[input] 'Problem input file'").default_value("input.txt"))
        .arg(Arg::from_usage("[p1] -1 --part1 'Solves Part 1'"))
        .arg(Arg::from_usage("[p2] -2 --part2 'Solves Part 2'").requires("special_reg"))
        .arg(Arg::from_usage(
            "[debug] -d --problem-debug 'Whether to execute the input program line by line'",
        ))
        .arg(
            Arg::from_usage(
                "[special_reg] -s --special-reg 'Which register in the input is the special one that is checked for equality in Part 2 problems'"
            ).default_value("5")
        )
        .arg(
            Arg::from_usage("[reg0] -0 --reg-0 'Overrides the value of register 0'")
                .takes_value(true)
                .conflicts_with_all(&["p1", "p2"]),
        )
        .get_matches();

    let input_filename = matches.value_of("input").unwrap();

    let code_str = fs::read_to_string(input_filename)?;
    let (ins_pointer, code) = parse_input(&code_str)?;

    let debug = matches.is_present("debug");
    let p1 = matches.is_present("p1");
    let p2 = matches.is_present("p2");
    let reg0 = matches.value_of("reg0").unwrap_or("0").parse()?;
    let special_reg = matches.value_of("special_reg").unwrap().parse::<usize>()?;

    let mut regs = vec![reg0, 0, 0, 0, 0, 0];

    let mut prev_special_regs = vec![];

    loop {
        let ins = match code.get(regs[ins_pointer]) {
            Some(ins) => ins,
            None => break,
        };

        if debug {
            println!("Executing {:?} at {}", ins, regs[ins_pointer]);
        }

        ins.execute(&mut regs);

        if debug {
            println!("{:?}", regs);
            std::io::stdin().read_line(&mut String::new()).unwrap();
        }

        if ins.name == "eqrr" {
            if p1 && prev_special_regs.len() == 0 {
                println!("Part 1: {:?}", regs[special_reg]);
                if !p2 {
                    break;
                }
            }

            if p2 && prev_special_regs.contains(&regs[special_reg]) {
                println!("Part 2: {:?}", prev_special_regs.last().unwrap());
                break;
            }

            if p1 || p2 {
                prev_special_regs.push(regs[special_reg]);
            }
        }

        regs[ins_pointer] += 1;
    }

    println!("Final registers: {:?}", regs);

    Ok(())
}

fn parse_input(code_str: &str) -> Result<(usize, Vec<Instruction>), anyhow::Error> {
    let mut code_lines = code_str.lines();

    let ins_pointer = code_lines
        .next()
        .map(|s| s.trim_start_matches("#ip "))
        .ok_or(anyhow!("Instruction pointer not found"))?
        .parse()?;

    let code = code_lines
        .map(|c| -> Result<Instruction, anyhow::Error> {
            let (op_str, inp1, inp2, output_reg) = c
                .split_whitespace()
                .collect_tuple()
                .ok_or(anyhow!("Instruction not in correct format"))?;

            let inp1: usize = inp1.parse()?;
            let inp2: usize = inp2.parse()?;
            let output_reg: usize = output_reg.parse()?;

            Ok(match op_str {
                "addr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::Add::add),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "addi" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::Add::add),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "mulr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::Mul::mul),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "muli" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::Mul::mul),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "banr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::BitAnd::bitand),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "bani" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::BitAnd::bitand),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "borr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::BitOr::bitor),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "bori" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(ops::BitOr::bitor),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "setr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, _| a),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "seti" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, _| a),
                    input: [Value::Imm(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "gtir" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a > b) as usize),
                    input: [Value::Imm(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "gtri" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a > b) as usize),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "gtrr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a > b) as usize),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "eqir" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a == b) as usize),
                    input: [Value::Imm(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                "eqri" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a == b) as usize),
                    input: [Value::Reg(inp1), Value::Imm(inp2)],
                    output_reg,
                },
                "eqrr" => Instruction {
                    name: op_str.to_string(),
                    operation: Box::new(|a, b| (a == b) as usize),
                    input: [Value::Reg(inp1), Value::Reg(inp2)],
                    output_reg,
                },
                _ => Err(anyhow!("Invalid operation"))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((ins_pointer, code))
}

struct Instruction {
    name: String,
    operation: Box<dyn Fn(usize, usize) -> usize>,
    input: [Value; 2],
    output_reg: usize,
}

impl Instruction {
    fn execute(&self, regs: &mut Vec<usize>) {
        regs[self.output_reg] = ops::Fn::call(
            &self.operation,
            self.input
                .iter()
                .map(|v| match v {
                    Value::Reg(r) => regs[*r],
                    Value::Imm(i) => *i,
                })
                .collect_tuple()
                .unwrap(),
        );
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {:?} {:?}",
            self.output_reg, self.name, self.input[0], self.input[1]
        )
    }
}

#[derive(Debug)]
enum Value {
    Reg(usize),
    Imm(usize),
}
