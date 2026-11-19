namespace Xmip.Operate;

/// <summary>
/// The two boundaries this build speaks: the module boundary a Module plugs
/// into and the operator boundary a surface drives from, versioned apart
/// (ADR-0027 clause 2) — abi's <c>boundaries</c>, asked through
/// <c>xmip_boundaries_v1</c> (<see cref="RuntimeWords.Boundaries"/>).
/// <c>xmip abi</c> renders the same answer and <c>Get-XmipAbi</c> emits it.
/// </summary>
/// <param name="ModuleVersion">The module handshake version.</param>
/// <param name="ModuleEntrypoint">The one symbol every module exports.</param>
/// <param name="ModuleLibraryFileName">How this platform names a loadable
/// module, shown for <c>xmip_core_transport_file</c>.</param>
/// <param name="OperateVersion">The operator boundary version.</param>
/// <param name="OperateEntrypoint">The one symbol a runtime exports for the
/// surfaces that watch.</param>
public sealed record AbiBoundaries(
    uint ModuleVersion,
    string ModuleEntrypoint,
    string ModuleLibraryFileName,
    uint OperateVersion,
    string OperateEntrypoint);
