using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// Section 20 of <c>include/xmip_operate.h</c>, crossed by P/Invoke: the
/// scripts that upgrade each database on a server from one schema version to
/// a later one, adding only (open problem 41; ADR-0015, amendment
/// 2026-10-20). The schema, its versions and the scripts are
/// <c>xmip-core-persist</c>'s; this binds the one call for
/// <c>xmip database</c> and <c>Get-XmipDatabaseScript</c>, which write
/// none of their own.
/// </summary>
/// <remarks>Pure in the header's sense: one instance serves a whole process
/// from any thread. A node's build only, as sections 10 to 16.</remarks>
public sealed class RuntimeDatabases
{
    private readonly delegate* unmanaged[Cdecl]<XmipStr, uint, uint, byte*, nuint, nuint*, int>
        _upgrade;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>
        _write;

    internal RuntimeDatabases(nint library)
    {
        _upgrade = (delegate* unmanaged[Cdecl]<XmipStr, uint, uint, byte*, nuint, nuint*, int>)
            NativeLibrary.GetExport(library, OperateAbi.DatabaseUpgradeEntrypoint);
        _write = RuntimeDesign.Export(library, OperateAbi.DatabaseWriteEntrypoint);
    }

    /// <summary>Section 20's symbols, which a node's runtime exports.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
        [OperateAbi.DatabaseUpgradeEntrypoint, OperateAbi.DatabaseWriteEntrypoint];

    /// <summary>
    /// The scripts of <paramref name="upgrade"/> written into
    /// <paramref name="folder"/>, made where it is not, none over a file —
    /// the paths written; or null with the runtime's sentence in
    /// <paramref name="refused"/> (<c>xmip_database_write_v1</c>, the rule
    /// <c>xmip database --out</c> writes by).
    /// </summary>
    public IReadOnlyList<string>? Write(
        DatabaseUpgrade upgrade, string folder, out string refused)
    {
        ArgumentNullException.ThrowIfNull(upgrade);
        ArgumentNullException.ThrowIfNull(folder);

        string scripts = new System.Text.Json.Nodes.JsonObject
        {
            ["version"] = upgrade.Version,
            ["scripts"] = new System.Text.Json.Nodes.JsonArray(
            [
                .. upgrade.Scripts.Select(script => (System.Text.Json.Nodes.JsonNode?)
                    new System.Text.Json.Nodes.JsonObject
                    {
                        ["name"] = script.Name,
                        ["text"] = script.Text,
                    }),
            ]),
        }.ToJsonString();

        if (!RuntimeDesign.Call(_write, scripts, folder, out string answer))
        {
            refused = answer;
            return null;
        }

        refused = string.Empty;
        using JsonDocument written = JsonDocument.Parse(answer);
        return
        [
            .. written.RootElement.EnumerateArray()
                .Select(path => path.GetString() ?? string.Empty),
        ];
    }

    /// <summary>
    /// The scripts that bring each database on <paramref name="server"/> —
    /// <c>postgresql</c>, <c>mssql</c> or <c>mariadb</c> — from schema version
    /// <paramref name="from"/> to <paramref name="to"/>, zero for this build's;
    /// or, refused, the runtime's sentence and no script.
    /// </summary>
    public DatabaseUpgrade Upgrade(string server, uint from, uint to)
    {
        ArgumentNullException.ThrowIfNull(server);

        byte[] word = Encoding.UTF8.GetBytes(server);
        byte[] room = new byte[RuntimeRemotes.Room];

        while (true)
        {
            nuint length = 0;
            int status;

            fixed (byte* named = word)
            fixed (byte* written = room)
            {
                unsafe
                {
                    status = _upgrade(
                        new XmipStr(named, (nuint)word.Length), from, to, written,
                        (nuint)room.Length, &length);
                }
            }

            if (length > (nuint)room.Length)
            {
                room = new byte[(int)length];
                continue;
            }

            string text = Encoding.UTF8.GetString(room, 0, (int)length);

            return status switch
            {
                0 => Answered(server, text),
                (int)XmipStatus.Invalid => new DatabaseUpgrade(server, 0, [], text),
                _ => new DatabaseUpgrade(
                    server, 0, [], $"the runtime answered with {(XmipStatus)status}"),
            };
        }
    }

    private static DatabaseUpgrade Answered(string server, string text)
    {
        using JsonDocument answer = JsonDocument.Parse(text);
        JsonElement root = answer.RootElement;

        return new DatabaseUpgrade(
            server,
            root.GetProperty("version").GetUInt32(),
            [.. root.GetProperty("scripts").EnumerateArray().Select(script => new DatabaseScript(
                script.GetProperty("name").GetString() ?? string.Empty,
                script.GetProperty("text").GetString() ?? string.Empty))],
            null);
    }
}
