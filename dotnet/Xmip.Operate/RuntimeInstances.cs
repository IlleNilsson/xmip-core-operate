using System.Runtime.InteropServices;
using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// Section 29 of <c>include/xmip_operate.h</c>, crossed by P/Invoke
/// (runtime-model.md section 22; ADR-0066, amendment 2026-11-25): the
/// Instances of each Work Process of the nodes running in this process, read
/// from the Ledger, and the Messages that failed in its queue. The Instance
/// is the runtime's; this binds the call once and decides none of it. A
/// Message that failed there is retried or dismissed as every failed Message
/// is, at a node's door through <see cref="RemoteNode.Act"/> and Xmip
/// Storage.
/// </summary>
/// <remarks>Pure in the header's sense: one instance serves a whole process
/// from any thread.</remarks>
public sealed class RuntimeInstances
{
    // A list fits in this, or is asked for again at its true length.
    private const int Room = 16 * 1024;

    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, uint, byte*, nuint, nuint*, int> _instances;

    internal RuntimeInstances(nint library)
    {
        _instances = (delegate* unmanaged[Cdecl]<
            XmipStr, XmipStr, uint, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.InstancesEntrypoint);
    }

    /// <summary>The symbols this binds, each of which a runtime must export;
    /// the publication's list is <see cref="PublicationReader"/>'s.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
    [
        OperateAbi.InstancesEntrypoint,
    ];

    /// <summary>
    /// The Instances of the Work Process <paramref name="workProcess"/>
    /// (empty: every one) of every node running in this process at or beneath
    /// <paramref name="node"/> (empty: every one), newest written first, at
    /// most <paramref name="most"/> of each (0: a hundred), and the Messages
    /// that failed in its queue.
    /// </summary>
    /// <exception cref="InvalidOperationException">Xmip Storage did not
    /// answer, in the runtime's words.</exception>
    public WorkProcessInstanceList Standing(string node, string workProcess, uint most)
    {
        ArgumentNullException.ThrowIfNull(node);
        ArgumentNullException.ThrowIfNull(workProcess);

        Utf8Pack pack = new([node, workProcess]);
        byte[] text = new byte[Room];
        nuint needed = 0;
        int status = Listed(pack, most, text, ref needed);

        if (needed > (nuint)text.Length)
        {
            text = new byte[(int)needed];
            status = Listed(pack, most, text, ref needed);
        }

        return status == 0
            ? WorkProcessInstanceList.Parse(text.AsMemory(0, (int)needed))
            : throw new InvalidOperationException(
                Encoding.UTF8.GetString(text, 0, (int)Math.Min(needed, (nuint)text.Length)));
    }

    private int Listed(Utf8Pack pack, uint most, byte[] text, ref nuint needed)
    {
        fixed (byte* data = pack.Bytes)
        fixed (byte* written = text)
        fixed (nuint* length = &needed)
        {
            unsafe
            {
                return _instances(
                    pack.Borrow(data, 0),
                    pack.Borrow(data, 1),
                    most,
                    written,
                    (nuint)text.Length,
                    length);
            }
        }
    }
}
