use solar::{
    ast::ast,
    interface::{diagnostics::EmittedDiagnostics, Ident, Session},
    parse::Parser,
};
use std::collections::HashSet;
use std::path::Path;

fn main() -> Result<(), EmittedDiagnostics> {
    let files = vec!["src/Counter.sol", "src/SelfDestruct.sol"];
    let sess = Session::builder()
        .with_buffer_emitter(solar::interface::ColorChoice::Auto)
        .build();

    for file_path in files {
        let path = Path::new(file_path);
        println!("\nAnalyzing file: {:?}", path);

        let _ = sess.enter(|| -> solar::interface::Result<()> {
            let arena = ast::Arena::new();
            let mut parser = Parser::from_file(&sess, &arena, path)?;
            let parsed_ast = parser.parse_file().unwrap(); 

            analyze_contract(parsed_ast).unwrap();
            Ok(())
        });
    }

    sess.emitted_diagnostics().unwrap_or_else(|| Ok(()))
}


fn analyze_contract(parsed_ast: ast::SourceUnit) -> Result<(), EmittedDiagnostics> {
    for item in parsed_ast.items {
        if let ast::ItemKind::Contract(ref contract) = item.kind {
            println!("Analyzing contract: {:?}", contract.name);

            if contract.name.to_string() == "Counter" {
                analyze_counter_contract(contract);
            }

            if contract.name.to_string() == "SelfDestruct" {
                analyze_selfdestruct_contract(contract);
            }
        }
    }
    Ok(())
}


fn analyze_counter_contract(contract: &ast::ItemContract) {
    let mut declared_variables = HashSet::new();
    let mut used_variables = HashSet::new();

    for contract_item in contract.body.iter() {
        if let ast::ItemKind::Variable(ref var) = contract_item.kind {
            if let Some(var_name) = &var.name {
                declared_variables.insert(var_name.clone());
            }
        }
    }

    for contract_item in contract.body.iter() {
        if let ast::ItemKind::Function(ref function) = contract_item.kind {
            if let Some(ref statements) = function.body {
                for stmt in statements.iter() {
                    collect_used_variables(stmt, &mut used_variables);
                    check_unsafe_arithmetic(stmt);
                }
            }
        }
    }

    for var in declared_variables.difference(&used_variables) {
        println!(
            "Warning: Unused variable in contract '{}': {:?}",
            contract.name, var
        );
    }
}

fn collect_used_variables(stmt: &ast::Stmt, used_variables: &mut HashSet<Ident>) {
    if let ast::StmtKind::Expr(ref expr) = stmt.kind {
        if let ast::ExprKind::Ident(ref ident) = expr.kind {
            used_variables.insert(ident.clone());
        }
    }
}


fn check_unsafe_arithmetic(stmt: &ast::Stmt) {
    if let ast::StmtKind::Expr(ref expr) = stmt.kind {
        if contains_div_before_mul(expr) {
            println!("Warning: Unsafe division before multiplication detected.");
        }
    }
}

fn contains_div_before_mul(expr: &ast::Expr) -> bool {
    match &expr.kind {
        ast::ExprKind::Binary(_, bin_op, right) if bin_op.kind == ast::BinOpKind::Div => {
            check_for_multiplication(right)
        }
        _ => false,
    }
}

fn check_for_multiplication(expr: &ast::Expr) -> bool {
    matches!(expr.kind, ast::ExprKind::Binary(_, bin_op, _) if bin_op.kind == ast::BinOpKind::Mul)
}


fn analyze_selfdestruct_contract(contract: &ast::ItemContract) {
    for contract_item in contract.body.iter() {
        if let ast::ItemKind::Function(ref function) = contract_item.kind {
            if let Some(ref statements) = function.body {
                if contains_unprotected_selfdestruct(statements) {
                    println!(
                        "Warning: Unprotected `selfdestruct` or `suicide` found in function '{}'.",
                        function
                            .header
                            .name
                            .as_ref()
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    );
                }
            }
        }
    }
}
fn contains_unprotected_selfdestruct(statements: &[ast::Stmt]) -> bool {
    let mut protection_found = false;

    for stmt in statements {
        match &stmt.kind {
     
            ast::StmtKind::Expr(ref expr) => match &expr.kind {
                ast::ExprKind::Call(ref func_expr, ref args) => {
                    if let ast::ExprKind::Ident(ref ident) = func_expr.kind {
                
                        if ident.to_string() == "selfdestruct" || ident.to_string() == "suicide" {
                  
                            if !protection_found {
                                println!("Warning: Unprotected selfdestruct or suicide detected.");
                                return true;
                            }
                        }
                    }

                  
                    if let ast::ExprKind::Ident(ref ident) = func_expr.kind {
                        if ident.to_string() == "require" {
                            protection_found = true;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    false
}
