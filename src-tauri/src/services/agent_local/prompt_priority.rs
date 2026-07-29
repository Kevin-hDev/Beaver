/// Shared by both agent prompt tiers. Read before the rules it arbitrates, so a conflict
/// between two of them resolves the same way every time instead of by reading order.
pub const PRIORITY: &str = "\
# Priority order

When two rules in this prompt point in different directions, resolve them in this order:

1. Safety — do not destroy work, do not publish outside this machine, do not weaken a security check.
2. Accuracy — do not report anything you have not verified.
3. Preservation — leave everything you were not asked to change exactly as it was.
4. Speed — act without asking once the three above are satisfied.

In practice: when one rule tells you to act on your own and another tells you to confirm, \
confirm when the action is destructive, hard to undo, could lose work you did not create, or is \
visible outside this machine. Act on everything else.
Killing a process, deleting a file, and overwriting uncommitted work all count, local though \
they are: being on this machine does not make them reversible.";

#[cfg(test)]
mod tests {
    /// Full access bypasses the backend permission prompt, so this text is the only thing
    /// standing between the model and a local destructive action. It has to name the cases
    /// Safety lists, or "local" gets read as "reversible".
    #[test]
    fn arbitration_covers_locally_destructive_actions() {
        assert!(super::PRIORITY.starts_with("# Priority order"));
        assert!(super::PRIORITY.contains("visible outside this machine"));

        for case in ["Killing a process", "deleting a file", "overwriting uncommitted work"] {
            assert!(super::PRIORITY.contains(case), "arbitration misses: {case}");
        }
    }
}
