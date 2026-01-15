//! Hatless Ralph - the constant coordinator.
//!
//! Ralph is always present, cannot be configured away, and acts as a universal fallback.

use crate::config::CoreConfig;
use crate::hat_registry::HatRegistry;
use ralph_proto::Topic;
use std::path::Path;

/// Hatless Ralph - the constant coordinator.
pub struct HatlessRalph {
    completion_promise: String,
    core: CoreConfig,
    hat_topology: Option<HatTopology>,
    /// Event to publish after coordination to start the hat workflow.
    starting_event: Option<String>,
}

/// Hat topology for multi-hat mode prompt generation.
pub struct HatTopology {
    hats: Vec<HatInfo>,
}

/// Information about a hat for prompt generation.
pub struct HatInfo {
    pub name: String,
    pub subscribes_to: Vec<String>,
    pub publishes: Vec<String>,
    pub instructions: String,
}

impl HatTopology {
    /// Creates topology from registry.
    pub fn from_registry(registry: &HatRegistry) -> Self {
        let hats = registry
            .all()
            .map(|hat| HatInfo {
                name: hat.name.clone(),
                subscribes_to: hat.subscriptions.iter().map(|t| t.as_str().to_string()).collect(),
                publishes: hat.publishes.iter().map(|t| t.as_str().to_string()).collect(),
                instructions: hat.instructions.clone(),
            })
            .collect();

        Self { hats }
    }
}

impl HatlessRalph {
    /// Creates a new HatlessRalph.
    ///
    /// # Arguments
    /// * `completion_promise` - String that signals loop completion
    /// * `core` - Core configuration (scratchpad, specs_dir, guardrails)
    /// * `registry` - Hat registry for topology generation
    /// * `starting_event` - Optional event to publish after coordination to start hat workflow
    pub fn new(
        completion_promise: impl Into<String>,
        core: CoreConfig,
        registry: &HatRegistry,
        starting_event: Option<String>,
    ) -> Self {
        let hat_topology = if registry.is_empty() {
            None
        } else {
            Some(HatTopology::from_registry(registry))
        };

        Self {
            completion_promise: completion_promise.into(),
            core,
            hat_topology,
            starting_event,
        }
    }

    /// Builds Ralph's prompt based on context.
    pub fn build_prompt(&self, context: &str) -> String {
        let mut prompt = self.core_prompt();

        // Include pending events BEFORE workflow so Ralph sees the task first
        if !context.trim().is_empty() {
            prompt.push_str("## PENDING EVENTS\n\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }

        prompt.push_str(&self.workflow_section());

        if let Some(topology) = &self.hat_topology {
            prompt.push_str(&self.hats_section(topology));
        }

        prompt.push_str(&self.event_writing_section());
        prompt.push_str(&self.done_section());

        prompt
    }

    /// Always returns true - Ralph handles all events as fallback.
    pub fn should_handle(&self, _topic: &Topic) -> bool {
        true
    }

    /// Checks if this is a fresh start (starting_event set, no scratchpad).
    ///
    /// Used to enable fast path delegation that skips the PLAN step
    /// when immediate delegation to specialized hats is appropriate.
    fn is_fresh_start(&self) -> bool {
        // Fast path only applies when starting_event is configured
        if self.starting_event.is_none() {
            return false;
        }

        // Check if scratchpad exists
        let path = Path::new(&self.core.scratchpad);
        !path.exists()
    }

    fn core_prompt(&self) -> String {
        let guardrails = self
            .core
            .guardrails
            .iter()
            .enumerate()
            .map(|(i, g)| format!("{}. {g}", 999 + i))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r"I'm Ralph. Fresh context each iteration.

### 0a. ORIENTATION
Study `{specs_dir}` to understand requirements.
Don't assume features aren't implemented—search first.

### 0b. SCRATCHPAD
Study `{scratchpad}`. It's shared state. It's memory.

Task markers:
- `[ ]` pending
- `[x]` done
- `[~]` cancelled (with reason)

### GUARDRAILS
{guardrails}

",
            scratchpad = self.core.scratchpad,
            specs_dir = self.core.specs_dir,
            guardrails = guardrails,
        )
    }

    fn workflow_section(&self) -> String {
        // Different workflow for solo mode vs multi-hat mode
        if self.hat_topology.is_some() {
            // Check for fast path: starting_event set AND no scratchpad
            if self.is_fresh_start() {
                // Fast path: immediate delegation without planning
                return format!(
                    r"## WORKFLOW

**FAST PATH**: Publish `{}` immediately to start the hat workflow.
Do not plan or analyze — delegate now.

",
                    self.starting_event.as_ref().unwrap()
                );
            }

            // Multi-hat mode: Ralph coordinates and delegates
            format!(
                r"## WORKFLOW

### 1. PLAN
Update `{scratchpad}` with prioritized tasks.

### 2. DELEGATE
Publish ONE event to hand off to specialized hats.

**CRITICAL: STOP after publishing the event.** A new iteration will start
with fresh context to handle the work. Do NOT continue working in this
iteration — let the next iteration handle the event with the appropriate
hat persona.

",
                scratchpad = self.core.scratchpad
            )
        } else {
            // Solo mode: Ralph does everything
            format!(
                r"## WORKFLOW

### 1. Study the prompt. 
Study, explore, and research what needs to be done. Use parallel subagents (up to 10) for searches.

### 2. PLAN
Update `{scratchpad}` with prioritized tasks.

### 3. IMPLEMENT
Pick ONE task. Only 1 subagent for build/tests.

### 4. COMMIT
Capture the why, not just the what. Mark `[x]` in scratchpad.

### 5. REPEAT
Until all tasks `[x]` or `[~]`.

",
                scratchpad = self.core.scratchpad
            )
        }
    }

    fn hats_section(&self, topology: &HatTopology) -> String {
        let mut section = String::from("## HATS\n\nDelegate via events.\n\n");

        // Include starting_event instruction if configured
        if let Some(ref starting_event) = self.starting_event {
            section.push_str(&format!(
                "**After coordination, publish `{}` to start the workflow.**\n\n",
                starting_event
            ));
        }

        // Build hat table
        section.push_str("| Hat | Triggers On | Publishes |\n");
        section.push_str("|-----|-------------|----------|\n");

        for hat in &topology.hats {
            let subscribes = hat.subscribes_to.join(", ");
            let publishes = hat.publishes.join(", ");
            section.push_str(&format!("| {} | {} | {} |\n", hat.name, subscribes, publishes));
        }

        section.push('\n');

        // Add instructions sections for hats with non-empty instructions
        for hat in &topology.hats {
            if !hat.instructions.trim().is_empty() {
                section.push_str(&format!("### {} Instructions\n\n", hat.name));
                section.push_str(&hat.instructions);
                if !hat.instructions.ends_with('\n') {
                    section.push('\n');
                }
                section.push('\n');
            }
        }

        section
    }

    fn event_writing_section(&self) -> String {
        format!(
            r#"## EVENT WRITING

Write events to `{events_file}` as:
{{"topic": "build.task", "payload": "...", "ts": "2026-01-14T12:00:00Z"}}

"#,
            events_file = ".agent/events.jsonl"
        )
    }

    fn done_section(&self) -> String {
        format!(
            r"## DONE

Output {} when all tasks complete.
",
            self.completion_promise
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RalphConfig;

    #[test]
    fn test_prompt_without_hats() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new(); // Empty registry
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Identity with ghuntley style
        assert!(prompt.contains("I'm Ralph. Fresh context each iteration."));

        // Numbered orientation phases
        assert!(prompt.contains("### 0a. ORIENTATION"));
        assert!(prompt.contains("Study"));
        assert!(prompt.contains("Don't assume features aren't implemented"));

        // Scratchpad section with task markers
        assert!(prompt.contains("### 0b. SCRATCHPAD"));
        assert!(prompt.contains("Task markers:"));
        assert!(prompt.contains("- `[ ]` pending"));
        assert!(prompt.contains("- `[x]` done"));
        assert!(prompt.contains("- `[~]` cancelled"));

        // Workflow with numbered steps (solo mode)
        assert!(prompt.contains("## WORKFLOW"));
        assert!(prompt.contains("### 1. Study the prompt"));
        assert!(prompt.contains("Use parallel subagents (up to 10)"));
        assert!(prompt.contains("### 2. PLAN"));
        assert!(prompt.contains("### 3. IMPLEMENT"));
        assert!(prompt.contains("Only 1 subagent for build/tests"));
        assert!(prompt.contains("### 4. COMMIT"));
        assert!(prompt.contains("Capture the why"));
        assert!(prompt.contains("### 5. REPEAT"));

        // Should NOT have hats section when no hats
        assert!(!prompt.contains("## HATS"));

        // Event writing and completion
        assert!(prompt.contains("## EVENT WRITING"));
        assert!(prompt.contains(".agent/events.jsonl"));
        assert!(prompt.contains("LOOP_COMPLETE"));
    }

    #[test]
    fn test_prompt_with_hats() {
        // Test multi-hat mode WITHOUT starting_event (no fast path)
        let yaml = r#"
hats:
  planner:
    name: "Planner"
    triggers: ["planning.start", "build.done", "build.blocked"]
    publishes: ["build.task"]
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done", "build.blocked"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        // Note: No starting_event - tests normal multi-hat workflow (not fast path)
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Identity with ghuntley style
        assert!(prompt.contains("I'm Ralph. Fresh context each iteration."));

        // Orientation phases
        assert!(prompt.contains("### 0a. ORIENTATION"));
        assert!(prompt.contains("### 0b. SCRATCHPAD"));

        // Multi-hat workflow: PLAN + DELEGATE, not IMPLEMENT
        assert!(prompt.contains("## WORKFLOW"));
        assert!(prompt.contains("### 1. PLAN"));
        assert!(prompt.contains("### 2. DELEGATE"), "Multi-hat mode should have DELEGATE step");
        assert!(
            !prompt.contains("### 3. IMPLEMENT"),
            "Multi-hat mode should NOT tell Ralph to implement"
        );
        assert!(
            prompt.contains("CRITICAL: STOP after publishing"),
            "Should explicitly tell Ralph to stop after publishing event"
        );

        // Hats section when hats are defined
        assert!(prompt.contains("## HATS"));
        assert!(prompt.contains("Delegate via events"));
        assert!(prompt.contains("| Hat | Triggers On | Publishes |"));

        // Event writing and completion
        assert!(prompt.contains("## EVENT WRITING"));
        assert!(prompt.contains("LOOP_COMPLETE"));
    }

    #[test]
    fn test_should_handle_always_true() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        assert!(ralph.should_handle(&Topic::new("any.topic")));
        assert!(ralph.should_handle(&Topic::new("build.task")));
        assert!(ralph.should_handle(&Topic::new("unknown.event")));
    }

    #[test]
    fn test_ghuntley_patterns_present() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Key ghuntley language patterns
        assert!(prompt.contains("Study"), "Should use 'study' verb");
        assert!(
            prompt.contains("Don't assume features aren't implemented"),
            "Should have 'don't assume' guardrail"
        );
        assert!(
            prompt.contains("parallel subagents"),
            "Should mention parallel subagents for reads"
        );
        assert!(
            prompt.contains("Only 1 subagent"),
            "Should limit to 1 subagent for builds"
        );
        assert!(
            prompt.contains("Capture the why"),
            "Should emphasize 'why' in commits"
        );

        // Numbered guardrails (999+)
        assert!(prompt.contains("### GUARDRAILS"), "Should have guardrails section");
        assert!(prompt.contains("999."), "Guardrails should use high numbers");
    }

    #[test]
    fn test_scratchpad_format_documented() {
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Task marker format is documented
        assert!(prompt.contains("- `[ ]` pending"));
        assert!(prompt.contains("- `[x]` done"));
        assert!(prompt.contains("- `[~]` cancelled (with reason)"));
    }

    #[test]
    fn test_starting_event_in_prompt() {
        // When starting_event is configured, prompt should include delegation instruction
        let yaml = r#"
hats:
  tdd_writer:
    name: "TDD Writer"
    triggers: ["tdd.start"]
    publishes: ["test.written"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            Some("tdd.start".to_string()),
        );

        let prompt = ralph.build_prompt("");

        // Should include delegation instruction
        assert!(
            prompt.contains("After coordination, publish `tdd.start` to start the workflow"),
            "Prompt should include starting_event delegation instruction"
        );
    }

    #[test]
    fn test_no_starting_event_instruction_when_none() {
        // When starting_event is None, no delegation instruction should appear
        let yaml = r#"
hats:
  some_hat:
    name: "Some Hat"
    triggers: ["some.event"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        // Should NOT include delegation instruction
        assert!(
            !prompt.contains("After coordination, publish"),
            "Prompt should NOT include starting_event delegation when None"
        );
    }

    #[test]
    fn test_hat_instructions_propagated_to_prompt() {
        // When a hat has instructions defined in config,
        // those instructions should appear in the generated prompt
        let yaml = r#"
hats:
  tdd_writer:
    name: "TDD Writer"
    triggers: ["tdd.start"]
    publishes: ["test.written"]
    instructions: |
      You are a Test-Driven Development specialist.
      Always write failing tests before implementation.
      Focus on edge cases and error handling.
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            Some("tdd.start".to_string()),
        );

        let prompt = ralph.build_prompt("");

        // Instructions should appear in the prompt
        assert!(
            prompt.contains("### TDD Writer Instructions"),
            "Prompt should include hat instructions section header"
        );
        assert!(
            prompt.contains("Test-Driven Development specialist"),
            "Prompt should include actual instructions content"
        );
        assert!(
            prompt.contains("Always write failing tests"),
            "Prompt should include full instructions"
        );
    }

    #[test]
    fn test_empty_instructions_not_rendered() {
        // When a hat has empty/no instructions, no instructions section should appear
        let yaml = r#"
hats:
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            None,
        );

        let prompt = ralph.build_prompt("");

        // No instructions section should appear for hats without instructions
        assert!(
            !prompt.contains("### Builder Instructions"),
            "Prompt should NOT include instructions section for hat with empty instructions"
        );
    }

    #[test]
    fn test_multiple_hats_with_instructions() {
        // When multiple hats have instructions, each should have its own section
        let yaml = r#"
hats:
  planner:
    name: "Planner"
    triggers: ["planning.start"]
    publishes: ["build.task"]
    instructions: "Plan carefully before implementation."
  builder:
    name: "Builder"
    triggers: ["build.task"]
    publishes: ["build.done"]
    instructions: "Focus on clean, testable code."
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            None,
        );

        let prompt = ralph.build_prompt("");

        // Both hats' instructions should appear
        assert!(
            prompt.contains("### Planner Instructions"),
            "Prompt should include Planner instructions section"
        );
        assert!(
            prompt.contains("Plan carefully before implementation"),
            "Prompt should include Planner instructions content"
        );
        assert!(
            prompt.contains("### Builder Instructions"),
            "Prompt should include Builder instructions section"
        );
        assert!(
            prompt.contains("Focus on clean, testable code"),
            "Prompt should include Builder instructions content"
        );
    }

    #[test]
    fn test_fast_path_with_starting_event() {
        // When starting_event is configured AND scratchpad doesn't exist,
        // should use fast path (skip PLAN step)
        let yaml = r#"
core:
  scratchpad: "/nonexistent/path/scratchpad.md"
hats:
  tdd_writer:
    name: "TDD Writer"
    triggers: ["tdd.start"]
    publishes: ["test.written"]
"#;
        let config: RalphConfig = serde_yaml::from_str(yaml).unwrap();
        let registry = HatRegistry::from_config(&config);
        let ralph = HatlessRalph::new(
            "LOOP_COMPLETE",
            config.core.clone(),
            &registry,
            Some("tdd.start".to_string()),
        );

        let prompt = ralph.build_prompt("");

        // Should use fast path - immediate delegation
        assert!(
            prompt.contains("FAST PATH"),
            "Prompt should indicate fast path when starting_event set and no scratchpad"
        );
        assert!(
            prompt.contains("Publish `tdd.start` immediately"),
            "Prompt should instruct immediate event publishing"
        );
        assert!(
            !prompt.contains("### 1. PLAN"),
            "Fast path should skip PLAN step"
        );
    }

    #[test]
    fn test_events_context_included_in_prompt() {
        // Given a non-empty events context
        // When build_prompt(context) is called
        // Then the prompt contains ## PENDING EVENTS section with the context
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let events_context = r#"[task.start] User's task: Review this code for security vulnerabilities
[build.done] Build completed successfully"#;

        let prompt = ralph.build_prompt(events_context);

        assert!(
            prompt.contains("## PENDING EVENTS"),
            "Prompt should contain PENDING EVENTS section"
        );
        assert!(
            prompt.contains("Review this code for security vulnerabilities"),
            "Prompt should contain the user's task"
        );
        assert!(
            prompt.contains("Build completed successfully"),
            "Prompt should contain all events from context"
        );
    }

    #[test]
    fn test_empty_context_no_pending_events_section() {
        // Given an empty events context
        // When build_prompt("") is called
        // Then no PENDING EVENTS section appears
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("");

        assert!(
            !prompt.contains("## PENDING EVENTS"),
            "Empty context should not produce PENDING EVENTS section"
        );
    }

    #[test]
    fn test_whitespace_only_context_no_pending_events_section() {
        // Given a whitespace-only events context
        // When build_prompt is called
        // Then no PENDING EVENTS section appears
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let prompt = ralph.build_prompt("   \n\t  ");

        assert!(
            !prompt.contains("## PENDING EVENTS"),
            "Whitespace-only context should not produce PENDING EVENTS section"
        );
    }

    #[test]
    fn test_events_section_before_workflow() {
        // Given events context with a task
        // When prompt is built
        // Then ## PENDING EVENTS appears BEFORE ## WORKFLOW
        let config = RalphConfig::default();
        let registry = HatRegistry::new();
        let ralph = HatlessRalph::new("LOOP_COMPLETE", config.core.clone(), &registry, None);

        let events_context = "[task.start] Implement feature X";
        let prompt = ralph.build_prompt(events_context);

        let events_pos = prompt.find("## PENDING EVENTS").expect("Should have PENDING EVENTS");
        let workflow_pos = prompt.find("## WORKFLOW").expect("Should have WORKFLOW");

        assert!(
            events_pos < workflow_pos,
            "PENDING EVENTS ({}) should come before WORKFLOW ({})",
            events_pos,
            workflow_pos
        );
    }
}
