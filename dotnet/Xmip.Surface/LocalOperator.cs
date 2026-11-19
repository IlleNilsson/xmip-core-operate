namespace Xmip.Surface;

/// <summary>
/// The surface over a node on this machine, reached at its local endpoint —
/// a named pipe on Windows, a Unix domain socket elsewhere — over gRPC
/// (ADR-0014, amendment 2026-10-13): a <see cref="ListenerOperator"/> with no
/// certificate, because the operating system admits only the node's account
/// and the machine's administrators to the endpoint and proves to the node
/// which user this process runs as. The node takes every act as that user.
/// Every .NET surface operates a node on its own machine this way; until
/// 2026-10-13 it loaded the runtime into its own process instead, which
/// operated only the nodes that process ran.
/// </summary>
/// <param name="endpoint">The node's local endpoint: a name — every node
/// serves one, <c>xmip-&lt;node&gt;</c> unless its <c>[operate] local</c> names
/// another — or the pipe's or socket's place. With none, a surface that
/// answers nothing and says why (<see cref="Unnamed"/>).</param>
/// <param name="within">How long each call waits before the node is shown
/// unreachable, "the node did not answer within" it; with none, what the node
/// declared from its <c>[tuning] local_timeout</c>, else 2 s (the runtime's
/// rule). The runtime's follow recovers it on its own.</param>
public sealed class LocalOperator(string? endpoint, TimeSpan within = default)
    : ListenerOperator(
        "LOCAL", endpoint ?? string.Empty,
        RuntimeLibrary.Rules.Surfaces.Local(endpoint ?? string.Empty, within))
{
    /// <summary>What every surface says when nothing names a node to operate
    /// on this machine, in the runtime's words.</summary>
    public static string Unnamed => RuntimeLibrary.Rules.Surfaces.Unnamed;

    /// <summary>What every surface says when <c>--local</c> or
    /// <c>-Local</c> names no endpoint, in the runtime's words; null when it
    /// names one.</summary>
    public static string? Refusal(string? endpoint)
    {
        string said = RuntimeLibrary.Rules.Surfaces.LocalRefusal(endpoint);
        return said.Length > 0 ? said : null;
    }
}
