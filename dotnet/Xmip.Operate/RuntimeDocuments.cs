using System.Text.Json.Nodes;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// Section 27 of <c>include/xmip_operate.h</c>, crossed by P/Invoke: a
/// configuration document read, written and one key of it set by
/// <c>xmip-core-configure</c>, the one reader the runtime reads its
/// configuration with (ADR-0031, amendment 2026-11-17). Every TOML document
/// a .NET surface reads or writes crosses here — its own, the Desktop's
/// preferences, a cluster's <c>cluster.xmip</c> — so .NET keeps no TOML
/// reader of its own; until 2026-11-17 it read them with a TOML package.
/// </summary>
/// <remarks>Pure in the header's sense: one instance serves a whole process
/// from any thread, a remote build's too. What crosses besides TOML text is
/// JSON, in memory only (ADR-0031 clause 2).</remarks>
public sealed class RuntimeDocuments
{
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, byte*, nuint, nuint*, int> _read;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, byte*, nuint, nuint*, int> _write;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, byte*, nuint, nuint*, int> _set;

    internal RuntimeDocuments(nint library)
    {
        _read = RuntimeDesign.Export(library, OperateAbi.DocumentReadEntrypoint);
        _write = RuntimeDesign.Export(library, OperateAbi.DocumentWriteEntrypoint);
        _set = RuntimeDesign.Export(library, OperateAbi.DocumentSetEntrypoint);
    }

    /// <summary>The symbols this binds, each of which a runtime must export,
    /// its remote build too.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
    [
        OperateAbi.DocumentReadEntrypoint,
        OperateAbi.DocumentWriteEntrypoint,
        OperateAbi.DocumentSetEntrypoint,
    ];

    /// <summary>The TOML <paramref name="text"/> as a JSON object: a table an
    /// object, an array an array, a date or time its TOML text. False, with
    /// the reader's sentence in <paramref name="answer"/>, when it is not
    /// TOML.</summary>
    public bool TryRead(string text, out string answer)
    {
        return RuntimeDesign.Call(_read, text, string.Empty, out answer);
    }

    /// <summary>The JSON object <paramref name="json"/> written as TOML.
    /// False, with the refusal, when it is no object or holds a
    /// null.</summary>
    public bool TryWrite(string json, out string answer)
    {
        return RuntimeDesign.Call(_write, json, string.Empty, out answer);
    }

    /// <summary>
    /// The TOML <paramref name="text"/> with the key <paramref name="key"/>
    /// of the table <paramref name="table"/> names from the root set to
    /// <paramref name="value"/>, or removed where it is null; every comment,
    /// blank line and order elsewhere as written. A table missing on the way
    /// is added when a value is set. False, with the refusal, when the text
    /// is not TOML or a key on the way holds a value.
    /// </summary>
    public bool TrySet(
        string text, IReadOnlyList<string> table, string key, JsonNode? value, out string answer)
    {
        ArgumentNullException.ThrowIfNull(table);
        ArgumentNullException.ThrowIfNull(key);

        string edit = new JsonObject
        {
            ["table"] = new JsonArray([.. table.Select(name => (JsonNode?)JsonValue.Create(name))]),
            ["key"] = key,
            ["value"] = value?.DeepClone(),
        }.ToJsonString();

        return RuntimeDesign.Call(_set, text, edit, out answer);
    }
}
