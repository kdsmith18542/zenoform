use pest::Parser as PestParser;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "zenoform.pest"]
pub struct ZenoformParser;

pub fn parse_dsl(source: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    let trimmed = source.trim();
    ZenoformParser::parse(Rule::program, trimmed)
}

pub fn compile_to_rust(source: &str) -> String {
    let pairs = match parse_dsl(source) {
        Ok(p) => p,
        Err(e) => return format!("// Parse Error: {}", e),
    };

    let mut code = String::from("// Generated Rust code\n\n");

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::module_def {
                    code.push_str(&gen_rust_module(inner));
                }
            }
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
        if block.as_rule() == Rule::input_block {
            // For MVP, we'll just assume inputs are available in the context
        }
        if block.as_rule() == Rule::cell_block {
            rust.push_str("    pub fn generate_cell(seed: i32, world_x: Fixed, world_y: Fixed) -> (u16, u8) {\n");
            for assignment in block.into_inner() {
                rust.push_str(&format!(
                    "        let {} = {};\n",
                    assignment.clone().into_inner().next().unwrap().as_str(),
                    gen_rust_expr(assignment.into_inner().nth(1).unwrap())
                ));
            }
            rust.push_str("        (height as u16, biome_id as u8)\n");
            rust.push_str("    }\n");
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
                        expr_code.push_str(term_inner.as_str());
                    }
                    Rule::function_call => {
                        let mut call_inner = term_inner.into_inner();
                        let func_name = call_inner.next().unwrap().as_str();
                        if func_name == "noise2d" {
                            expr_code.push_str("zenoform_core::noise::value_noise_2d(");
                            let args: Vec<String> = call_inner.map(|arg| gen_rust_expr(arg)).collect();
                            expr_code.push_str(&args.join(", "));
                            expr_code.push(')');
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
        if block.as_rule() == Rule::cell_block {
            cairo.push_str("    fn generate_cell(seed: i128, world_x: Fixed, world_y: Fixed) -> (u16, u8) {\n");
            for assignment in block.into_inner() {
                cairo.push_str(&format!(
                    "        let {} = {};\n",
                    assignment.clone().into_inner().next().unwrap().as_str(),
                    gen_cairo_expr(assignment.into_inner().nth(1).unwrap())
                ));
            }
            cairo.push_str("        (height.try_into().unwrap(), biome_id.try_into().unwrap())\n");
            cairo.push_str("    }\n");
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
                            expr_code.push(')');
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

    let mut code = String::from("# Generated Mojo code\n\n");
    code.push_str("from algorithm import vectorize, parallelize, StaticVector\n");

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::module_def {
                    let mut module_inner = inner.into_inner();
                    let name = module_inner.next().unwrap().as_str().replace(".", "_");

                    code.push_str(&format!("\nfn generate_{}(\n", name));
                    code.push_str("    seed: Int,\n");
                    code.push_str("    chunk_x: Int,\n");
                    code.push_str("    chunk_y: Int,\n");
                    code.push_str("    width: Int,\n");
                    code.push_str("    height: Int,\n");
                    code.push_str(") -> List[Cell]:\n");
                    code.push_str("    var cells = List[Cell]()\n");
                    code.push_str("    var seed_int = Simd[DType.int32, 1](seed)\n\n");

                    for block in module_inner {
                        if block.as_rule() == Rule::cell_block {
                            code.push_str("    @parameter\n");
                            code.push_str("    fn generate_cell(world_x: Int, world_y: Int) -> Cell:\n");
                            for assignment in block.into_inner() {
                                let mut assign_inner = assignment.into_inner();
                                let var = assign_inner.next().unwrap().as_str();
                                let expr = assign_inner.next().unwrap();
                                code.push_str(&format!("        var {} = {}\n", var, gen_mojo_expr(expr)));
                            }
                            code.push_str(
                                "        return Cell(height, temperature, moisture, biome_id, resource_mask)\n",
                            );
                        }
                    }

                    code.push_str("\n    for y in range(height):\n");
                    code.push_str("        for x in range(width):\n");
                    code.push_str("            cells.append(generate_cell(\n");
                    code.push_str("                chunk_x * width + x,\n");
                    code.push_str("                chunk_y * height + y\n");
                    code.push_str("            ))\n");
                    code.push_str("    return cells\n");
                }
            }
        }
    }

    code
}

fn gen_mojo_expr(pair: Pair<Rule>) -> String {
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
                            expr_code.push_str("simd_noise_2d(");
                            let args: Vec<String> = call_inner.map(|arg| gen_mojo_expr(arg)).collect();
                            expr_code.push_str(&args.join(", "));
                            expr_code.push(')');
                        } else {
                            expr_code.push_str(&format!("{}(", func_name));
                            let args: Vec<String> = call_inner.map(|arg| gen_mojo_expr(arg)).collect();
                            expr_code.push_str(&args.join(", "));
                            expr_code.push(')');
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_module() {
        let source = r#"module terrain.fixed_noise.v1 {
    input:
        seed: Field
        chunk_x: i32
    cell:
        height = noise2d(seed, world_x, world_y)
    output:
        height: u16
}"#;
        let result = parse_dsl(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_module() {
        let source = "not a valid module { }";
        let result = parse_dsl(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_to_rust() {
        let source = r#"module terrain.test {
    input:
        seed: Field
    cell:
        height = noise2d(seed, world_x, world_y)
    output:
        height: u16
}"#;
        let output = compile_to_rust(source);
        assert!(output.contains("pub mod"));
        assert!(output.contains("generate_cell"));
        assert!(output.contains("value_noise_2d"));
    }

    #[test]
    fn test_compile_to_cairo() {
        let source = r#"module terrain.test {
    input:
        seed: Field
    cell:
        height = noise2d(seed, world_x, world_y)
    output:
        height: u16
}"#;
        let output = compile_to_cairo(source);
        assert!(output.contains("mod"));
        assert!(output.contains("generate_cell"));
        assert!(output.contains("value_noise_2d"));
    }

    #[test]
    fn test_compile_to_mojo() {
        let source = r#"module terrain.test {
    input:
        seed: Field
    cell:
        height = noise2d(seed, world_x, world_y)
    output:
        height: u16
}"#;
        let output = compile_to_mojo(source);
        assert!(output.contains("# Generated Mojo"));
        assert!(output.contains("fn generate_terrain_test"));
        assert!(output.contains("simd_noise_2d"));
    }

    #[test]
    fn test_parse_malformed_module() {
        let result = parse_dsl("module foo { missing stuff }");
        assert!(result.is_err());
    }

    #[test]
    fn test_rust_codegen_handles_parse_error() {
        let output = compile_to_rust("invalid!!!");
        assert!(output.starts_with("// Parse Error"));
    }

    #[test]
    fn test_cairo_codegen_handles_parse_error() {
        let output = compile_to_cairo("invalid!!!");
        assert!(output.starts_with("// Parse Error"));
    }

    #[test]
    fn test_mojo_codegen_handles_parse_error() {
        let output = compile_to_mojo("invalid!!!");
        assert!(output.starts_with("// Parse Error"));
    }
}
