using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The upgrade scripts the runtime wrote (<see cref="RuntimeDatabases"/>, section
/// 20), written once for every surface into the folder an operator names:
/// <c>xmip database --out</c> and <c>Get-XmipDatabaseScript -Path</c>
/// (ADR-0015, amendment 2026-10-20). Nothing is overwritten: where any script
/// is there already, none is written and the refusal names it.
/// </summary>
public static class DatabaseScripts
{
    /// <summary>
    /// Each of <paramref name="upgrade"/>'s scripts written into
    /// <paramref name="folder"/>, made where it is missing: the paths
    /// written, in the order to run them; or null with
    /// <paramref name="refused"/> naming the file already there.
    /// </summary>
    public static IReadOnlyList<string>? Write(
        DatabaseUpgrade upgrade, string folder, out string refused)
    {
        // The refusal and the writing are the runtime's
        // (xmip_database_write_v1), the ones xmip database --out writes by.
        return RuntimeLibrary.Rules.Databases.Write(upgrade, folder, out refused);
    }
}
