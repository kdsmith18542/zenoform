use pest_derive::Parser;
use pest::Parser as PestParser;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "zenoform.pest"]
pub struct ZenoformParser;

pub fn parse_dsl(source: &str) -> Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> {
    ZenoformParser::parse(Rule::program, source)
}

pub fn compile_to_rust(source: &str) -> String {
    let pairs = match parse_dsl(source) {
        Ok(p) => p,
        Err(e) => return format!("// Parse Error: {}", e),
    };

    let mut code = String::from("// Generated Rust code\n\n");
    
    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::module_def {
                        code.push_str(&gen_rust_module(inner));
                    }
                }
            }
            _ => {}
        }
    }

    code
}

fn gen_rust_module(pair: Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    
    let mut rust = format!("pub mod {} {{\n", name.replace(".", "_"));
    rust.push_str("    use zenoform_core::fixed::{Fixed, FixedTrait};\n\n");

    for block in inner {
        match block.as_rule() {
            Rule::input_block => {
                // For MVP, we'll just assume inputs are available in the context
            }
            Rule::cell_block => {
                rust.push_str("    pub fn generate_cell(seed: i32, world_x: Fixed, world_y: Fixed) -> (u16, u8) {\n");
                for assignment in block.into_inner() {
                    rust.push_str(&format!("        let {} = {};\n", 
                        assignment.clone().into_inner().next().unwrap().as_str(),
                        gen_rust_expr(assignment.into_inner().nth(1).unwrap())
                    ));
                }
                rust.push_str("        (height as u16, biome_id as u8)\n");
                rust.push_str("    }\n");
            }
            _ => {}
        }
    }

    rust.push_str("}\n");
    rust
}

fn gen_rust_expr(pair: Pair<Rule>) -> String {
    let mut expr_code = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::term => {
                let term_inner = inner.into_inner().next().unwrap();
                match term_inner.as_rule() {
                    Rule::number => expr_code.push_str(term_inner.as_str()),
                    Rule::identifier => {
                        let id = term_inner.as_str();
                        if id == "world_x" || id == "world_y" || id == "seed" {
                             expr_code.push_str(id);
                        } else {
                             expr_code.push_str(id);
                        }
                    }
                    Rule::function_call => {
                        let mut call_inner = term_inner.into_inner();
                        let func_name = call_inner.next().unwrap().as_str();
                        if func_name == "noise2d" {
                            expr_code.push_str("zenoform_core::noise::value_noise_2d(");
                            let args: Vec<String> = call_inner.map(|arg| gen_rust_expr(arg)).collect();
                            expr_code.push_str(&args.join(", "));
                            expr_code.push_str(")");
                        }
                    }
                    _ => expr_code.push_str(term_inner.as_str()),
                }
            }
            Rule::operation => expr_code.push_str(&format!(" {} ", inner.as_str())),
            _ => {}
        }
    }
    expr_code
}

pub fn compile_to_cairo(source: &str) -> String {
    let pairs = match parse_dsl(source) {
        Ok(p) => p,
        Err(e) => return format!("// Parse Error: {}", e),
    };

    let mut code = String::from("// Generated Cairo code\n\n");
    
    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::module_def {
                    code.push_str(&gen_cairo_module(inner));
                }
            }
        }
    }

    code
}

fn gen_cairo_module(pair: Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().replace(".", "_");
    
    let mut cairo = format!("mod {} {{\n", name);
    cairo.push_str("    use zenoform_terrain_v1::fixed::{Fixed, FixedTrait};\n\n");

    for block in inner {
        match block.as_rule() {
            Rule::cell_block => {
                cairo.push_str("    fn generate_cell(seed: i128, world_x: Fixed, world_y: Fixed) -> (u16, u8) {\n");
                for assignment in block.into_inner() {
                    cairo.push_str(&format!("        let {} = {};\n", 
                        assignment.clone().into_inner().next().unwrap().as_str(),
                        gen_cairo_expr(assignment.into_inner().nth(1).unwrap())
                    ));
                }
                cairo.push_str("        (height.try_into().unwrap(), biome_id.try_into().unwrap())\n");
                cairo.push_str("    }\n");
            }
            _ => {}
        }
    }

    cairo.push_str("}\n");
    cairo
}

fn gen_cairo_expr(pair: Pair<Rule>) -> String {
    let mut expr_code = String::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::term => {
                let term_inner = inner.into_inner().next().unwrap();
                match term_inner.as_rule() {
                    Rule::number => expr_code.push_str(term_inner.as_str()),
                    Rule::identifier => expr_code.push_str(term_inner.as_str()),
                    Rule::function_call => {
                        let mut call_inner = term_inner.into_inner();
                        let func_name = call_inner.next().unwrap().as_str();
                        if func_name == "noise2d" {
                            expr_code.push_str("zenoform_terrain_v1::noise::value_noise_2d(");
                            let args: Vec<String> = call_inner.map(|arg| gen_cairo_expr(arg)).collect();
                            expr_code.push_str(&args.join(", "));
                            expr_code.push_str(")");
                        }
                    }
                    _ => expr_code.push_str(term_inner.as_str()),
                }
            }
            Rule::operation => expr_code.push_str(&format!(" {} ", inner.as_str())),
            _ => {}
        }
    }
    expr_code
}

pub fn compile_to_mojo(source: &str) -> String {
    let pairs = match parse_dsl(source) {
        Ok(p) => p,
        Err(e) => return format!("// Parse Error: {}", e),
    };

    let mut code = String::from("// Generated Mojo code\n\n");
    
    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::module_def {
                    code.push_str(&format!("fn generate_{}(seed: Int32, world_x: SIMD[DType.int32, 8], world_y: SIMD[DType.int32, 8]):\n", inner.into_inner().next().unwrap().as_str().replace(".", "_")));
                    code.push_str("    # SIMD optimized procedural logic here\n");
                    code.push_str("    pass\n");
                }
            }
        }
    }

    code
}
