namespace Xmip.Surface;

/// <summary>
/// What an operator stated for one invocation about which surface to read,
/// over what the host's document says: a node on another machine to follow,
/// a node on this one to follow, a published snapshot to read, and the
/// runtime library to load. A cmdlet states it as <c>-Remote</c>,
/// <c>-Local</c>, <c>-Snapshot</c> and <c>-Library</c>, as the command line
/// states the first three, <c>--remote</c>, <c>--local</c> and
/// <c>--snapshot</c>, to the same rule in Rust; the
/// prompt as the snapshot <c>Start-XmipTest</c> told it to follow. Blank is
/// no statement. <see cref="SurfaceChoice.Stated"/> is the one precedence
/// over it.
/// </summary>
/// <param name="Remote">A node on another machine, at its operate address
/// (ADR-0014, amendment 2026-10-12).</param>
/// <param name="Snapshot">One published snapshot, one cluster (ADR-0052,
/// amendment 2026-09-20).</param>
/// <param name="Runtime">The runtime library, over the discovery rule: the
/// one the gRPC client and every call with no node behind it are made
/// through. It names no surface.</param>
/// <param name="Local">A node on this machine, at its local endpoint
/// (ADR-0014, amendment 2026-10-13).</param>
public sealed record SurfaceLine(
    string? Remote = null, string? Snapshot = null, string? Runtime = null,
    string? Local = null)
{
    /// <summary>Nothing stated: the document decides.</summary>
    public static SurfaceLine None { get; } = new();

    /// <summary>Why the line cannot be obeyed, or null when it can: a blank
    /// remote is no statement, and what an address that names something is,
    /// the runtime judges as it opens it (<see cref="RemoteOperator.Refusal"/>
    /// says what a <c>--remote</c> naming nothing is told). Asked before the
    /// runtime library the line states is loaded, so it asks the runtime
    /// nothing.</summary>
    public string? Refusal => null;
}
