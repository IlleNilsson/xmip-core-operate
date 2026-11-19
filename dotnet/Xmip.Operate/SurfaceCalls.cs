using System.Runtime.InteropServices;
using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>Section 28.2's calls on an open surface, bound once for every
/// <see cref="SurfaceNode"/> the library opens.</summary>
internal sealed class SurfaceCalls
{
    // Room for an answer before asking again at its length; an act is never
    // asked again, so its answer is read in this room.
    internal const int Room = 64 * 1024;

    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _source;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _standing;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _act;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _actMessage;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _control;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _failed;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _secrets;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> _setSecret;
    private readonly delegate* unmanaged[Cdecl]<
        nint, ulong, nint*, nint*, byte*, nuint, nuint*, int> _read;

    internal SurfaceCalls(nint library)
    {
        Free = (delegate* unmanaged[Cdecl]<nint, void>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceFreeEntrypoint);
        Answers = (delegate* unmanaged[Cdecl]<nint, byte*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceAnswersEntrypoint);
        _read = (delegate* unmanaged[Cdecl]<nint, ulong, nint*, nint*, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceReadEntrypoint);
        Wait = (delegate* unmanaged[Cdecl]<nint, ulong, uint, ulong*, ulong*, int>)
            NativeLibrary.GetExport(library, OperateAbi.SurfaceWaitEntrypoint);
        _source = Asking(library, OperateAbi.SurfaceSourceEntrypoint);
        _standing = Asking(library, OperateAbi.SurfaceStandingEntrypoint);
        _act = Asking(library, OperateAbi.SurfaceActEntrypoint);
        _actMessage = Asking(library, OperateAbi.SurfaceActMessageEntrypoint);
        _control = Asking(library, OperateAbi.SurfaceControlEntrypoint);
        _failed = Asking(library, OperateAbi.SurfaceFailedEntrypoint);
        _secrets = Asking(library, OperateAbi.SurfaceSecretsEntrypoint);
        _setSecret = Asking(library, OperateAbi.SurfaceSetSecretEntrypoint);
    }

    internal delegate* unmanaged[Cdecl]<nint, void> Free { get; }

    internal delegate* unmanaged[Cdecl]<nint, byte*, int> Answers { get; }

    // The surface's reading, written where the pointers say; their room is
    // the caller's.
    internal unsafe int Read(
        nint surface, ulong seen, nint* publication, nint* index, byte* written, nuint cap,
        nuint* length)
    {
        unsafe
        {
            return _read(surface, seen, publication, index, written, cap, length);
        }
    }

    internal delegate* unmanaged[Cdecl]<nint, ulong, uint, ulong*, ulong*, int> Wait { get; }

    internal int Source(nint surface, out string said)
    {
        return Asked(_source, surface, 0, string.Empty, string.Empty, again: true, out said);
    }

    internal int Standing(nint surface, out string said)
    {
        return Asked(_standing, surface, 0, string.Empty, string.Empty, again: true, out said);
    }

    internal int Act(nint surface, ulong caller, string asked, out string said)
    {
        return Asked(_act, surface, caller, asked, string.Empty, again: false, out said);
    }

    internal int ActMessage(nint surface, ulong caller, string asked, out string said)
    {
        return Asked(_actMessage, surface, caller, asked, string.Empty, again: false, out said);
    }

    internal int Control(nint surface, string asked, out string said)
    {
        return Asked(_control, surface, 0, asked, string.Empty, again: false, out said);
    }

    internal int Failed(nint surface, string asked, out string said)
    {
        return Asked(_failed, surface, 0, asked, string.Empty, again: true, out said);
    }

    internal int Secrets(nint surface, ulong caller, string node, out string said)
    {
        return Asked(_secrets, surface, caller, node, string.Empty, again: true, out said);
    }

    internal int SetSecret(
        nint surface, ulong caller, string asked, string value, out string said)
    {
        return Asked(_setSecret, surface, caller, asked, value, again: false, out said);
    }

    private static delegate* unmanaged[Cdecl]<
        nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> Asking(
        nint library, string entrypoint)
    {
        return (delegate* unmanaged[Cdecl]<
            nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, entrypoint);
    }

    // One asking: the answer in the room, asked again at its length only
    // where `again` says asking twice changes nothing; the value packed
    // beside the rest is cleared once it crossed.
    private static int Asked(
        delegate* unmanaged[Cdecl]<nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> ask,
        nint surface,
        ulong caller,
        string input,
        string argument,
        bool again,
        out string said)
    {
        Utf8Pack pack = new([input, argument]);
        byte[] text = new byte[Room];
        nuint needed = 0;
        int status = Once(ask, surface, caller, pack, text, ref needed);

        if (again && needed > (nuint)text.Length)
        {
            text = new byte[(int)needed];
            status = Once(ask, surface, caller, pack, text, ref needed);
        }

        Array.Clear(pack.Bytes);
        said = Encoding.UTF8.GetString(text, 0, (int)Math.Min(needed, (nuint)text.Length));
        return status;
    }

    private static int Once(
        delegate* unmanaged[Cdecl]<nint, ulong, XmipStr, XmipStr, byte*, nuint, nuint*, int> ask,
        nint surface,
        ulong caller,
        Utf8Pack pack,
        byte[] text,
        ref nuint needed)
    {
        nuint length = 0;
        int status;

        fixed (byte* data = pack.Bytes)
        fixed (byte* written = text)
        {
            unsafe
            {
                status = ask(
                    surface, caller, pack.Borrow(data, 0), pack.Borrow(data, 1), written,
                    (nuint)text.Length, &length);
            }
        }

        needed = length;
        return status;
    }
}
