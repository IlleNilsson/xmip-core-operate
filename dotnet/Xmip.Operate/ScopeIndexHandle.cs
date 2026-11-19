using System.Runtime.InteropServices;

namespace Xmip.Operate;

/// <summary>
/// A publication's scope tree the runtime holds (section 28.1,
/// <c>XmipScopeIndex</c>): immutable, asked from any thread, and released
/// once nothing holds it — an index is shared by every view of one
/// publication and never disposed by any of them, so the collector frees it.
/// </summary>
public sealed class ScopeIndexHandle : SafeHandle
{
    private readonly delegate* unmanaged[Cdecl]<nint, void> _free;

    internal ScopeIndexHandle(nint index, delegate* unmanaged[Cdecl]<nint, void> free)
        : base(0, ownsHandle: true)
    {
        _free = free;
        SetHandle(index);
    }

    /// <summary>Whether it holds no index.</summary>
    public override bool IsInvalid => handle == 0;

    /// <inheritdoc />
    protected override bool ReleaseHandle()
    {
        unsafe
        {
            _free(handle);
        }

        return true;
    }
}
