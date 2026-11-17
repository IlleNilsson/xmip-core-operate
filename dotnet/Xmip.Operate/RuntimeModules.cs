using System.Runtime.InteropServices;
using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// Section 26 of <c>include/xmip_operate.h</c>, crossed by P/Invoke: a
/// module library probed through the runtime's one loader, the one a node
/// opens its modules with, and judged by the one rule a node judges a
/// descriptor by (ADR-0012, amendment 2026-11-17). Until 2026-11-17 .NET
/// opened the library itself and judged it by a rule of its own; this binds
/// the export once for every .NET surface and keeps nothing else.
/// </summary>
/// <remarks>Any thread; one instance serves a whole process. A node's build
/// only (<see cref="RemoteBuild"/>).</remarks>
public sealed class RuntimeModules
{
    // A probe's answer is a few hundred bytes and the lines a module logs;
    // a longer one is asked for again at its true length, which probes the
    // library again.
    private const int Room = 64 * 1024;

    private readonly delegate* unmanaged[Cdecl]<XmipStr, byte*, nuint, nuint*, int> _probe;

    internal RuntimeModules(nint library)
    {
        _probe = (delegate* unmanaged[Cdecl]<XmipStr, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.ModuleProbeEntrypoint);
    }

    /// <summary>The symbols this binds, each of which a runtime must
    /// export.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } = [OperateAbi.ModuleProbeEntrypoint];

    /// <summary>
    /// Open the library at <paramref name="library"/>, ask it for one
    /// instance, read what it declares and its last error, destroy it and
    /// unload it. A library that cannot be asked is an answer, not an
    /// exception: <see cref="ModuleProbed.Unloadable"/> says why.
    /// </summary>
    public ModuleProbed Probe(string library)
    {
        ArgumentNullException.ThrowIfNull(library);

        using PinnedStr path = XmipStr.Pin(library);
        byte[] answer = new byte[Room];
        nuint needed = 0;
        int status = Once(path, answer, ref needed);

        if (needed > (nuint)answer.Length)
        {
            answer = new byte[(int)needed];
            status = Once(path, answer, ref needed);
        }

        return status == 0
            ? ModuleProbed.Read(Encoding.UTF8.GetString(answer, 0, (int)needed))
            : throw new InvalidOperationException(
                $"the runtime answered a probe with {(XmipStatus)status}");
    }

    // The path is pinned by the caller until both calls return.
    private int Once(PinnedStr path, byte[] answer, ref nuint needed)
    {
        fixed (byte* written = answer)
        fixed (nuint* length = &needed)
        {
            unsafe
            {
                return _probe(path.Value, written, (nuint)answer.Length, length);
            }
        }
    }
}
