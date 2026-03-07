//! Deployment node printing.

use sruja_language::DeploymentNode;

pub fn print_deployment(out: &mut String, deployment: &DeploymentNode, indent: usize) {
    let indent_str = "  ".repeat(indent);
    out.push_str(&indent_str);
    out.push_str("deployment ");
    out.push_str(&deployment.id);
    if let Some(label) = &deployment.label {
        out.push_str(" \"");
        out.push_str(label);
        out.push('"');
    }
    if let Some(tech) = &deployment.technology {
        out.push_str(&format!(" \"{}\"", tech));
    }
    if !deployment.children.is_empty() {
        out.push_str(" {\n");
        for child in &deployment.children {
            print_deployment(out, child, indent + 1);
        }
        out.push_str(&indent_str);
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}
