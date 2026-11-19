using System.Runtime.InteropServices;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>Section 28.2 of <c>include/xmip_operate.h</c>: a surface — a
/// published snapshot or a node's operate listener — opened, read, followed
/// and asked to act; <c>operate::surface</c>, the one the command line reads
/// in Rust (ADR-0052, amendment 2026-11-18). Opening never fails: a surface
/// that reaches nothing says why on every answer.</summary>
public sealed class RuntimeSurfaces
{
    private readonly delegate* unmanaged[Cdecl]<XmipStr, uint, nint*, int> _snapshot;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, uint, nint*, int> _local;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, XmipStr, XmipStr, nint*, int> _remote;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, XmipStr, XmipStr,
        delegate* unmanaged[Cdecl]<void*, XmipStr, byte*, nuint, nuint*, int>, void*, nint*,
        int> _remoteKept;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _decided;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _named;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _unlisted;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _portAt;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _remoteRefusal;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _localRefusal;

    internal RuntimeSurfaces(nint library)
    {
        _snapshot = (delegate* unmanaged[Cdecl]<XmipStr, uint, nint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceSnapshotEntrypoint);
        _local = (delegate* unmanaged[Cdecl]<XmipStr, uint, nint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceLocalEntrypoint);
        _remote = (delegate* unmanaged[Cdecl]<XmipStr, XmipStr, XmipStr, XmipStr, nint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceRemoteEntrypoint);
        _remoteKept = (delegate* unmanaged[Cdecl]<
            XmipStr, XmipStr, XmipStr, XmipStr,
            delegate* unmanaged[Cdecl]<void*, XmipStr, byte*, nuint, nuint*, int>, void*, nint*,
            int>)NativeLibrary.GetExport(library, OperateAbi.SurfaceRemoteKeptEntrypoint);
        _decided = RuntimeDesign.Export(library, OperateAbi.SurfaceDecidedEntrypoint);
        _named = RuntimeDesign.Export(library, OperateAbi.SurfaceNamedEntrypoint);
        _unlisted = RuntimeDesign.Export(library, OperateAbi.SurfaceUnlistedEntrypoint);
        _portAt = RuntimeDesign.Export(library, OperateAbi.SurfacePortAtEntrypoint);
        _remoteRefusal = RuntimeDesign.Export(library, OperateAbi.SurfaceRemoteRefusalEntrypoint);
        _localRefusal = RuntimeDesign.Export(library, OperateAbi.SurfaceLocalRefusalEntrypoint);
        Calls = new SurfaceCalls(library);
        NotActed = Said(RuntimeDesign.Export(library, OperateAbi.SurfaceNotActedEntrypoint));
        Unnamed = Said(RuntimeDesign.Export(library, OperateAbi.SurfaceUnnamedEntrypoint));
    }

    /// <summary>The symbols it binds, each of which a runtime exports, its
    /// remote build too.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
    [
        OperateAbi.SurfaceSnapshotEntrypoint,
        OperateAbi.SurfaceLocalEntrypoint,
        OperateAbi.SurfaceRemoteEntrypoint,
        OperateAbi.SurfaceRemoteKeptEntrypoint,
        OperateAbi.SurfaceFreeEntrypoint,
        OperateAbi.SurfaceSourceEntrypoint,
        OperateAbi.SurfaceAnswersEntrypoint,
        OperateAbi.SurfaceStandingEntrypoint,
        OperateAbi.SurfaceReadEntrypoint,
        OperateAbi.SurfaceWaitEntrypoint,
        OperateAbi.SurfaceActEntrypoint,
        OperateAbi.SurfaceActMessageEntrypoint,
        OperateAbi.SurfaceControlEntrypoint,
        OperateAbi.SurfaceFailedEntrypoint,
        OperateAbi.SurfaceSecretsEntrypoint,
        OperateAbi.SurfaceSetSecretEntrypoint,
        OperateAbi.SurfaceDecidedEntrypoint,
        OperateAbi.SurfaceNamedEntrypoint,
        OperateAbi.SurfaceUnlistedEntrypoint,
        OperateAbi.SurfacePortAtEntrypoint,
        OperateAbi.SurfaceRemoteRefusalEntrypoint,
        OperateAbi.SurfaceLocalRefusalEntrypoint,
        OperateAbi.SurfaceNotActedEntrypoint,
        OperateAbi.SurfaceUnnamedEntrypoint,
    ];

    /// <summary>Why a snapshot takes no act, in the runtime's words.</summary>
    public string NotActed { get; }

    /// <summary>What a local surface naming no node says on every answer.</summary>
    public string Unnamed { get; }

    internal SurfaceCalls Calls { get; }

    /// <summary>The surface over the publication at <paramref name="path"/>.</summary>
    public SurfaceNode Snapshot(string path)
    {
        return Opened(_snapshot, path, 0);
    }

    /// <summary>The surface over the node on this machine at its local
    /// endpoint <paramref name="endpoint"/> — none where it is empty — each
    /// call waiting at most <paramref name="within"/>, zero for what the node
    /// declared.</summary>
    public SurfaceNode Local(string endpoint, TimeSpan within = default)
    {
        uint bound = (uint)Math.Clamp(within.TotalMilliseconds, 0, uint.MaxValue);
        return Opened(_local, endpoint, bound);
    }

    /// <summary>The surface over the node at <paramref name="address"/>,
    /// presenting the PEM files the paths name — null where none is — and
    /// trusting the anchors there.</summary>
    public SurfaceNode Remote(
        string address, string? certificate, string? key, string? anchors)
    {
        ArgumentNullException.ThrowIfNull(address);
        Utf8Pack pack = new([address, certificate ?? "", key ?? "", anchors ?? ""]);
        nint handle = 0;

        fixed (byte* data = pack.Bytes)
        {
            unsafe
            {
                Check(_remote(
                    pack.Borrow(data, 0), pack.Borrow(data, 1), pack.Borrow(data, 2),
                    pack.Borrow(data, 3), &handle));
            }
        }

        return new SurfaceNode(Calls, handle);
    }

    /// <summary>The surface over the node at <paramref name="address"/>,
    /// presenting the chain at <paramref name="certificate"/>, whose key the
    /// keeper called <paramref name="keeper"/> holds: <paramref name="sign"/>
    /// is asked every signature — given what is signed, ECDSA on P-256 with
    /// SHA-256, DER; it throws to refuse — from any thread, until the surface
    /// is disposed; trusting the anchors at <paramref name="anchors"/>.</summary>
    public SurfaceNode RemoteKept(
        string address, string certificate, string? anchors, string keeper,
        Func<byte[], byte[]> sign)
    {
        ArgumentNullException.ThrowIfNull(address);
        ArgumentNullException.ThrowIfNull(certificate);
        ArgumentNullException.ThrowIfNull(keeper);
        ArgumentNullException.ThrowIfNull(sign);
        Utf8Pack pack = new([address, certificate, anchors ?? "", keeper]);
        GCHandle kept = GCHandle.Alloc(sign);
        nint handle = 0;

        fixed (byte* data = pack.Bytes)
        {
            unsafe
            {
                Check(_remoteKept(
                    pack.Borrow(data, 0), pack.Borrow(data, 1), pack.Borrow(data, 2),
                    pack.Borrow(data, 3), &KeeperSigning.Sign, (void*)GCHandle.ToIntPtr(kept),
                    &handle));
            }
        }

        return new SurfaceNode(Calls, handle, kept);
    }

    /// <summary>The surface <paramref name="asked"/> names over the host's
    /// <c>[Xmip]</c> table <paramref name="table"/>, both JSON: true with
    /// the choice in <paramref name="answer"/>, false with the refusal.</summary>
    public bool Decided(string asked, string table, out string answer)
    {
        return RuntimeDesign.Call(_decided, asked, table, out answer);
    }

    /// <summary>The files a remote surface presents, <paramref name="asked"/>
    /// as JSON, checked: true with
    /// <c>{"certificate","key","anchors","kept"}</c>, false with the
    /// refusal.</summary>
    public bool Named(string asked, out string answer)
    {
        return RuntimeDesign.Call(_named, asked, string.Empty, out answer);
    }

    /// <summary>The files the host's <c>[Xmip]</c> table
    /// <paramref name="table"/> names, else the environment
    /// <paramref name="asked"/> hands — <c>{"base","environment","kept"}</c>,
    /// both JSON — checked: true with
    /// <c>{"certificate","key","anchors","kept"}</c>, false with the
    /// refusal.</summary>
    public bool Named(string asked, string table, out string answer)
    {
        ArgumentNullException.ThrowIfNull(table);
        return RuntimeDesign.Call(_named, asked, table, out answer);
    }

    /// <summary>What a list of failed Messages that is none says, asked as
    /// JSON; empty for a list.</summary>
    public string Unlisted(string asked)
    {
        _ = RuntimeDesign.Call(_unlisted, asked, string.Empty, out string said);
        return said;
    }

    /// <summary>The node and Send Port <paramref name="scope"/> is at, as
    /// <c>{"node","port"}</c>.</summary>
    public string PortAt(string scope)
    {
        _ = RuntimeDesign.Call(_portAt, scope, string.Empty, out string said);
        return said;
    }

    /// <summary>Why <c>--remote</c> with <paramref name="address"/> after it
    /// is refused; empty where it is not.</summary>
    public string RemoteRefusal(string? address)
    {
        _ = RuntimeDesign.Call(
            _remoteRefusal, address ?? string.Empty, string.Empty, out string said);
        return said;
    }

    /// <summary>Why <c>--local</c> with <paramref name="endpoint"/> after it
    /// is refused; empty where it is not.</summary>
    public string LocalRefusal(string? endpoint)
    {
        _ = RuntimeDesign.Call(
            _localRefusal, endpoint ?? string.Empty, string.Empty, out string said);
        return said;
    }

    private SurfaceNode Opened(
        delegate* unmanaged[Cdecl]<XmipStr, uint, nint*, int> open, string place, uint within)
    {
        ArgumentNullException.ThrowIfNull(place);
        Utf8Pack pack = new([place]);
        nint handle = 0;

        fixed (byte* data = pack.Bytes)
        {
            unsafe
            {
                Check(open(pack.Borrow(data, 0), within, &handle));
            }
        }

        return new SurfaceNode(Calls, handle);
    }

    private static string Said(
        delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> export)
    {
        _ = RuntimeDesign.Call(export, string.Empty, string.Empty, out string said);
        return said;
    }

    private static void Check(int status)
    {
        if (status != 0)
        {
            throw new InvalidOperationException(
                $"the runtime answered a surface's opening with {(XmipStatus)status}");
        }
    }
}
