namespace Xmip.Operate;

/// <summary>
/// What a node declared of itself (ADR-0056) as the runtime read it: from
/// the capability record the node published
/// (<see cref="RuntimeRules.Published"/>, <c>observe::capability</c>) or
/// from a run's entry for it (<see cref="RuntimeRules.Entry"/>,
/// <c>node::Capability</c>).
/// </summary>
/// <param name="Node">The node's name, read as a name and nothing else.</param>
/// <param name="Roles">The roles it declared, in declaration order; none
/// when refused.</param>
/// <param name="Capabilities">Its Node capabilities, in declaration order,
/// <c>online</c> among them where it may assume the internet (ADR-0056,
/// amendment 2026-11-24); a run's entry never says, and a refused
/// declaration says nothing.</param>
/// <param name="Refusal">Why the declaration was refused, in
/// <c>node::NodeRole::declared</c>'s words, or empty (ADR-0055).</param>
public sealed record DeclaredCapability(
    string Node, IReadOnlyList<string> Roles, IReadOnlyList<string> Capabilities, string Refusal);
