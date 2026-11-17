using System.Text.Json;
using Xmip.Abi.Module;

namespace Xmip.Operate;

/// <summary>
/// What a module library said when the runtime's one loader opened it and
/// asked, or why it could not be asked: section 26 of
/// <c>include/xmip_operate.h</c>, the one object <c>xmip probe</c> renders
/// and <c>Get-XmipModuleDescriptor</c> emits (ADR-0012, amendment
/// 2026-11-17). Opened and judged in the runtime, by the rule a node judges
/// a module by; nothing here judges.
/// </summary>
/// <param name="Library">The library probed, as given.</param>
/// <param name="Status">What the entrypoint returned; <c>NotFound</c> where
/// the library could not be asked.</param>
/// <param name="Provider">The descriptor's provider.</param>
/// <param name="Module">The descriptor's module name.</param>
/// <param name="Standard">The descriptor's standard, empty for core.</param>
/// <param name="AbiVersion">The abi_version the module says it speaks.</param>
/// <param name="TraitVersion">The trait's major.minor.</param>
/// <param name="ModuleVersion">The module's major.minor.patch.</param>
/// <param name="LastError">The module's last error.</param>
/// <param name="Unloadable">Why the library could not be asked — not there,
/// no library for this platform, or no entrypoint; empty when it was.</param>
/// <param name="Refusal">Why the host refuses it, one sentence; empty when
/// nothing is wrong.</param>
/// <param name="Conforms">It loaded, made an instance, and nothing is
/// refused.</param>
/// <param name="Log">Each line the module logged while it was held, in
/// order.</param>
public sealed record ModuleProbed(
    string Library,
    XmipStatus Status,
    string Provider,
    string Module,
    string Standard,
    uint AbiVersion,
    string TraitVersion,
    string ModuleVersion,
    string LastError,
    string Unloadable,
    string Refusal,
    bool Conforms,
    IReadOnlyList<string> Log)
{
    /// <summary>Whether the library loaded and its entrypoint was
    /// asked.</summary>
    public bool Loaded => Unloadable.Length == 0;

    /// <summary>The status as one line an operator reads.</summary>
    public string Meaning => Status.Explain();

    /// <summary>The header's JSON, read.</summary>
    internal static ModuleProbed Read(string json)
    {
        using JsonDocument document = JsonDocument.Parse(json);
        JsonElement said = document.RootElement;

        return new ModuleProbed(
            Text(said, "library"),
            (XmipStatus)said.GetProperty("status").GetInt32(),
            Text(said, "provider"),
            Text(said, "module"),
            Text(said, "standard"),
            said.GetProperty("abi_version").GetUInt32(),
            Text(said, "trait_version"),
            Text(said, "module_version"),
            Text(said, "last_error"),
            Text(said, "unloadable"),
            Text(said, "refusal"),
            said.GetProperty("conforms").GetBoolean(),
            [.. said.GetProperty("log").EnumerateArray().Select(line => line.GetString() ?? "")]);
    }

    private static string Text(JsonElement said, string name)
    {
        return said.GetProperty(name).GetString() ?? string.Empty;
    }
}
