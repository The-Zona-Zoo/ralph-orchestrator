//! Instruction builder for prepending orchestration context to prompts.

use ralph_proto::Hat;

/// Builds the prepended instructions for agent prompts.
#[derive(Debug)]
pub struct InstructionBuilder {
    completion_promise: String,
}

impl InstructionBuilder {
    /// Creates a new instruction builder.
    pub fn new(completion_promise: impl Into<String>) -> Self {
        Self {
            completion_promise: completion_promise.into(),
        }
    }

    /// Builds single-hat mode instructions.
    pub fn build_single_hat(&self, prompt_content: &str) -> String {
        format!(
            r#"ORCHESTRATION CONTEXT:
You are running within the Ralph Orchestrator loop. This system will call you
repeatedly for multiple iterations until the overall task is complete.

IMPORTANT INSTRUCTIONS:
1. Implement only ONE small, focused task per iteration
2. Mark subtasks complete as you finish them (update PROMPT.md checkboxes)
3. Commit your changes after each iteration for checkpointing
4. Use .agent/workspace/ for temporary files

WORKFLOW:
- Explore: Research and understand the codebase
- Plan: Design your implementation approach
- Implement: Write tests first (TDD), then code
- Commit: Commit changes with clear messages

AGENT SCRATCHPAD:
Before starting, check .agent/scratchpad.md for previous progress.
At iteration end, update it with:
- What you accomplished
- What remains to be done
- Any blockers or decisions made

Do NOT restart from scratch if scratchpad shows progress.

COMPLETION:
When ALL tasks in PROMPT.md are complete, output:
{promise}

---
ORIGINAL PROMPT:
{prompt}"#,
            promise = self.completion_promise,
            prompt = prompt_content
        )
    }

    /// Builds multi-hat mode instructions for a specific hat.
    pub fn build_multi_hat(&self, hat: &Hat, events_context: &str) -> String {
        let mut instructions = String::new();

        instructions.push_str("ORCHESTRATION CONTEXT:\n");
        instructions.push_str(&format!("You are the {} agent in a multi-agent system.\n\n", hat.name));

        if !hat.instructions.is_empty() {
            instructions.push_str("YOUR ROLE:\n");
            instructions.push_str(&hat.instructions);
            instructions.push_str("\n\n");
        }

        instructions.push_str("EVENT COMMUNICATION:\n");
        instructions.push_str("Use <event> tags to communicate with other agents:\n");
        instructions.push_str(r#"<event topic="your.topic">Your message</event>"#);
        instructions.push_str("\n\n");

        if !hat.publishes.is_empty() {
            instructions.push_str("You typically publish to: ");
            let topics: Vec<&str> = hat.publishes.iter().map(|t| t.as_str()).collect();
            instructions.push_str(&topics.join(", "));
            instructions.push_str("\n\n");
        }

        instructions.push_str(&format!(
            "COMPLETION:\nWhen the overall task is complete, output:\n{}\n\n",
            self.completion_promise
        ));

        instructions.push_str("---\nINCOMING EVENTS:\n");
        instructions.push_str(events_context);

        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_hat_instructions() {
        let builder = InstructionBuilder::new("LOOP_COMPLETE");
        let instructions = builder.build_single_hat("Implement feature X");

        assert!(instructions.contains("LOOP_COMPLETE"));
        assert!(instructions.contains("Implement feature X"));
        assert!(instructions.contains("AGENT SCRATCHPAD"));
    }

    #[test]
    fn test_multi_hat_instructions() {
        let builder = InstructionBuilder::new("DONE");
        let hat = Hat::new("impl", "Implementer")
            .with_instructions("Write clean, tested code.");

        let instructions = builder.build_multi_hat(&hat, "Event: task.start - Begin work");

        assert!(instructions.contains("Implementer agent"));
        assert!(instructions.contains("Write clean, tested code"));
        assert!(instructions.contains("DONE"));
        assert!(instructions.contains("task.start"));
    }
}
