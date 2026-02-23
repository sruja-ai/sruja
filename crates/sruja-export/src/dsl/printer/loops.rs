//! Feedback and causal loop printing.

use sruja_language::{CausalLoop, FeedbackLoop};

use super::relation;

pub fn print_feedback_loop(out: &mut String, loop_data: &FeedbackLoop) {
    out.push_str(loop_data.id.as_str());
    out.push_str(" = feedback \"");
    out.push_str(&loop_data.title);
    out.push_str("\" {\n");
    out.push_str(&format!("  loop_type \"{}\"\n", loop_data.loop_type));
    if let Some(loop_id) = &loop_data.loop_id {
        out.push_str(&format!("  loop_id \"{}\"\n", loop_id));
    }
    if let Some(desc) = &loop_data.description {
        out.push_str(&format!("  description \"{}\"\n", desc));
    }
    for rel in &loop_data.relationships {
        relation::print_relation(out, rel, 1);
    }
    out.push_str("}\n");
}

pub fn print_causal_loop(out: &mut String, loop_data: &CausalLoop) {
    out.push_str(loop_data.id.as_str());
    out.push_str(" = causal_loop \"");
    out.push_str(&loop_data.title);
    out.push_str("\" {\n");
    out.push_str(&format!("  loop_type \"{}\"\n", loop_data.loop_type));
    if let Some(loop_id) = &loop_data.loop_id {
        out.push_str(&format!("  loop_id \"{}\"\n", loop_id));
    }
    if let Some(desc) = &loop_data.description {
        out.push_str(&format!("  description \"{}\"\n", desc));
    }
    for rel in &loop_data.relationships {
        out.push_str("  ");
        out.push_str(&rel.from);
        out.push_str(" -> ");
        out.push_str(&rel.to);
        out.push_str(" {\n");
        if let Some(effect) = &rel.effect {
            out.push_str(&format!("    effect \"{}\"\n", effect));
        }
        out.push_str(&format!("    polarity \"{}\"\n", rel.polarity));
        if let Some(delay) = &rel.delay {
            out.push_str(&format!("    delay \"{}\"\n", delay));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
}
